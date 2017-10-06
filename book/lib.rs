//@ # Writing a Static Analyser in Rust
//@
//@ To try out the concept of [*Literate Programming*][lp] (using the awesome [tango] 
//@ crate), I'm going to write a small static analyser for a basic Programming 
//@ language. Because it's so much more interesting to use a programming language
//@ available in the wild, compared to some contrived example, we're going to 
//@ analyse Delphi (a Pascal variant).
//@
//@ Believe it or not, but this entire book is the actual source code for this
//@ static analyser. Check out the [repo] on GitHub if you want to see more.
//@
//@ [repo]: https://github.com/Michael-F-Bryan/static-analyser-in-rust
//@ [tango]: https://github.com/pnkfelix/tango
//@ [lp]: https://en.wikipedia.org/wiki/Literate_programming
//@
//@ Here's your basic Hello World:
//@
//@ ```pascal
//@ procedure TForm1.ShowAMessage;
//@ begin
//@   ShowMessage('Hello World!');
//@ end;
//@ ```
//@
//@ All you need to do is use the IDE to hook that function up to be run whenever
//@ a button is clicked and it'll show a "Hello World!" dialog.
//@
//@ > **Note:** The API docs for this crate should be placed alongside the book.
//@ > You can access then [here](../doc/static_analyser/index.html) (you'll need
//@ > to use `cargo doc --open` if viewing locally).
//@
//@
//@ First up, lets add some top-level docs and import some crates we're going to 
//@ need:

//! A parser and static analysis library for exploring Delphi code.
//!
//! This is written using a *Literate Programming* style, so you may find it
//! easier to inspect the [rendered version] instead.
//!
//! [rendered version]: https://michael-f-bryan.github.io/static-analyser-in-rust/

#![deny(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;

//@ There are several steps you need to perform to do static analysis, first is 
//@ tokenizing (often called *lexical analysis*). This stage turns the characters
//@ in the raw source code into `Tokens` like "if", "begin", integer literals and
//@ operators.

pub mod lex;

//@ Next we do the parsing stage (*semantic analysis*) to convert our stream of 
//@ tokens into an Abstract Syntax Tree. This is a representation of the program
//@ as it exists on disk. 
//@
//@ A lot of static analysis passes can get away with working purely at the AST
//@ level. For example, if you want to make sure people don't accidentally divide
//@ by zero it's just a case of looking for all division nodes and checking that
//@ the right hand isn't a numeric literal representing zero (e.g. `0` or `0.0`).
//@
//@ Another useful lint which can be applied at this level is 
//@ [cyclomatic complexity], i.e. how "complex" a function/procedure is. This is
//@ normally just a case of walking the body of a function and counting the number
//@ of branches, loops, and `try/catch` blocks encountered.
//@
//@ [cyclomatic complexity]: https://en.wikipedia.org/wiki/Cyclomatic_complexity

#[macro_use]
pub mod parse;

//@ The third step is type checking and generating a High level Intermediate 
//@ Representation (HIR), often referred to as "lowering" (converting from a high
//@ level representation to a lower one). 
//@
//@  While the AST is very flexible and useful, it works at the language syntax
//@  level and completely misses the *semantics* of a language. This means an
//@  expression like `'foo' + 42` or dereferencing a float is a perfectly valid
//@  AST node.
//@  
//@  To perform some of the more advanced analyses we'll need to have access to
//@  the full context surrounding an expression to determine if it is valid. This
//@  typically involves figuring out the type for each variable and expression,
//@  as well as resolving imports and stitching multiple unit files into one
//@  cohesive data structure.

pub mod lowering;

//@ Now we've *finally* resolved all imports and types we're *guaranteed* to have
//@ a syntactically and semantically valid program. This doesn't mean it's correct
//@ though! At this stage we can create passes which employ the full strength of
//@ the compiler/static analyser to check the *logic* of our program. This lets
//@ us do 

pub mod analysis;

//@ We also need to handle internal errors. To keep things clean lets put that in
//@ its own module too.

pub mod errors;

//@ Another very important thing to have is a mapping which lets you talk about a
//@ logical chunk of code (i.e. *this* function body or *that* string literal) and
//@ retrieve the corresponding source code. This will be crucial for the later 
//@ steps where we want to indicate where an error occurred to the user.

pub mod codemap;

//@ Finally, there's the `Driver`. He's in charge of the show an is usually the
//@ thing you'll want to invoke or hook into to tweak the analysis process.

mod driver;
pub use driver::Driver;


//@ ## A Note on Project Design
//@
//@ A lot of the time, if you need to write a parser you'll want to use some sort
//@ of parser combinator or generator library. This greatly decreases the effort
//@ and time required, but you often trade that off with poor error handling and
//@ error messages. Because we're writing a tool for analysing your code, it stands
//@ to reason that if the user passes in dodgy code, we can detect this (without
//@ crashing) and emit a **useful** error message. All of this means that we'll
//@ want to write the lexing and parsing stuff by hand instead of deferring to 
//@ another tool.
//@
//@ If you are following along at home, click through to one of the sections to 
//@ learn about it in more detail.
