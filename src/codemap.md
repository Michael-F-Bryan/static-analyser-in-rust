# The CodeMap

A `CodeMap` gives you a central mapping from spans to their location in the
group of files being analysed.

As usual, lets add in a couple imports and module-level documentation.

```rust
//! A mapping from arbitrary locations and sections of source code to their
//! contents.

use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
```

We start off with a `Span`. This is really just a wrapper around an integer,
with the assumption that a span will **always** correspond to something in
the `CodeMap`. This means using a span from one `CodeMap` with another will
result in a panic if you are lucky, or silently give you garbage.

```rust
/// A unique identifier pointing to a substring in some file.
///
/// To get back the original string this points to you'll need to look it up
/// in a `CodeMap` or `FileMap`. 
#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub struct Span(usize);
```

For our purposes, the `CodeMap` will just contain a list of `FileMap`s. These
keep track of their name, contents, and the mapping of spans to locations in
that content.

```rust
/// A mapping of `Span`s to the files in which they are located.
#[derive(Debug)]
pub struct CodeMap {
    counter: Rc<AtomicUsize>,
    files: Vec<Rc<FileMap>>,
}

/// A mapping which keeps track of a file's contents and allows you to cheaply
/// access substrings of the original content.
#[derive(Clone, Debug)]
pub struct FileMap {
    name: String,
    contents: String,
    counter: Rc<AtomicUsize>,
    items: RefCell<HashMap<Span, Range<usize>>>
}
```

The codemap has a couple useful methods for adding new files and looking up the
string corresponding to a span.

```rust
impl CodeMap {
    /// Create a new, empty `CodeMap`.
    pub fn new() -> CodeMap {
        let counter = Rc::new(AtomicUsize::new(0));
        let files = Vec::new();
        CodeMap { counter, files }
    }

    /// Add a new file to the `CodeMap` and get back a reference to it.
    pub fn insert_file<C, F>(&mut self, filename: F, contents: C) -> Rc<FileMap> 
    where F: Into<String>,
          C: Into<String>,
    {
        let filemap = FileMap {
            name: filename.into(),
            contents: contents.into(),
            items: RefCell::new(HashMap::new()),
            counter: self.counter.clone(),
        };
        let fm = Rc::new(filemap);
        self.files.push(fm.clone());

        fm
    }

    /// Get the substring that this `Span` corresponds to.
    pub fn lookup(&self, span: Span) -> &str {
        for filemap in &self.files {
            if let Some(substr) = filemap.lookup(span) {
                return substr;
            }
        }

        panic!("Tried to lookup {:?} but it wasn't in any of the FileMaps... This is a bug!")
    }
}
```

You may have noticed that `FileMap` contains a `RefCell<HashMap<_>>`. This is 
because we want to pass around multiple pointers to a file mapping, yet still
be able to add new spans if we want to. It also contains a reference to the
parent `CodeMap`'s counter so when we insert new spans into the `FileMap` 
they'll still get globally unique IDs.

```rust
impl FileMap {
    /// Get the name of this `FileMap`.
    pub fn filename(&self) -> &str {
        &self.name
    }

    /// Get the entire content of this file.
    pub fn contents(&self) -> &str {
        &self.contents
    }

    /// Lookup a span in this `FileMap`.
    ///
    /// # Panics
    ///
    /// If the `FileMap`'s `items` hashmap contains a span, but that span 
    /// **doesn't** point to a valid substring this will panic. If you ever
    /// get into a situation like this then things are almost certainly FUBAR.
    pub fn lookup(&self, span: Span) -> Option<&str> {
        let range = match self.range_of(span) {
            Some(r) => r,
            None => return None,
        };

        match self.contents.get(range.clone()) {
            Some(substr) => Some(substr),
            None => panic!("FileMap thinks it contains {:?}, \
            but the range ({:?}) doesn't point to anything valid!", span, range),
        }
    }

    /// Get the range corresponding to this span.
    pub fn range_of(&self, span: Span) -> Option<Range<usize>> {
        self.items.borrow().get(&span).cloned() 
    }
}
```

Users can freely add new spans to a `FileMap`, to do this we'll take in the 
start and end indices, create a new span ID by incrementing our counter, then
we insert the new span and range into the `items`. In debug builds we'll do 
bounds checks, but it's an assumption that the `start` and `end` indices are
both within bounds, and lie on valid codepoint boundaries.

```rust
impl FileMap {
    /// Ask the `FileMap` to give you the span corresponding to the half-open
    /// interval `[start, end)`.
    ///
    /// # Panics
    ///
    /// In debug mode, this will panic if either `start` or `end` are outside
    /// the source code or if they don't lie on a codepoint boundary.
    ///
    /// It is assumed that the `start` and `indices` were originally obtained
    /// from the file's contents.
    pub fn insert_span(&self, start: usize, end: usize) -> Span {
        debug_assert!(self.contents.is_char_boundary(start), 
            "Start doesn't lie on a char boundary");
        debug_assert!(self.contents.is_char_boundary(end), 
            "End doesn't lie on a char boundary");
        debug_assert!(start < self.contents.len(), 
            "Start lies outside the content string");
        debug_assert!(end < self.contents.len(), 
            "End lies outside the content string");

        let range = start..end;
        let span_id = self.counter.fetch_add(1, Ordering::Relaxed);
        let span = Span(span_id);

        self.items.borrow_mut().insert(span, range);
        span
    }
}
```