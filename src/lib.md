# Writing a Static Analyser in Rust

To try out the concept of *Literate Programming* (using the awesome [tango] 
crate), I'm going to write a small static analyser for a basic Programming 
language. Because it's so much more interesting to use a programming language
available in the wild, compared to some contrived example, we're going to 
analyse Delphi (a Pascal variant).

Here's your basic Hello World:

```pascal
procedure TForm1.ShowAMessage;
begin
  ShowMessage('Hello World!');
end;
```

> **Note:** The API docs for this crate should be placed alongside the book.
> You can access then [here](../doc/static_analyser/index.html).


First up, lets import some crates we're going to need:

```rust
#[macro_use]
extern crate error_chain;
```

There are several steps you need to perform to do static analysis, first is 
tokenizing (often called *lexical analysis*).

```rust
pub mod lex;
```

Next is parsing (*semantic analysis*).

```rust
pub mod parse;
```

The third step is type checking.

```rust
pub mod typeck;
```

Now *finally* we've got everything set up to do the static analysis.

```rust
pub mod analysis;
```

We also need to handle internal errors. To keep things clean lets put that in
its own module too.

```rust
pub mod errors;
```


## A Note on Project Design

A lot of the time, if you need to write a parser you'll want to use some sort
of parser combinator or generator library. This greatly decreases the effort
and time required, but you often trade that off with poor error handling and
error messages. Because we're writing a tool for analysing your code, it stands
to reason that if the user passes in dodgy code, we can detect this (without
crashing) and emit a **useful** error message. All of this means that we'll
want to write the lexing and parsing stuff by hand instead of deferring to 
another tool.

If you are following along at home, click through to one of the sections to 
learn about it in more detail.
