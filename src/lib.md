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