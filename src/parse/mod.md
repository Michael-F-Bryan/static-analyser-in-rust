# The Parsing Stage

The Core part of the parsing stage is our `Parser`. This is what actually
converts the tokens from a single file into an Abstract Syntax Tree which
we can analyse.

```rust
//! Parse a stream of `Tokens` into an *Abstract Syntax Tree* we can use for
//! the later steps.
#![allow(missing_docs, dead_code, unused_imports)]

#[macro_use]
mod macros;
mod parser;
pub use self::parser::Parser;
```

The other important datastructure in this module is the Abstract Syntax Tree
and its various types of nodes.

```rust
mod ast;
pub use self::ast::{Literal, LiteralKind};
```

If you are following along at home you'll probably want to keep the pages for
both the `Parser` and the `AST` open in tabs so you can swap between them 
easily.