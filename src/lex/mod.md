# Lexical Analysis

Before anything else, lets import some things we'll require.

```rust
use std::str;
use errors::*;
```

A lexer's job is to turn normal strings (which a human can read) into 
something more computer-friendly called a `Token`. A `Token` can be multiple
different types representing multiple different things, so it makes sense to 
use a Rust enum here.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(usize),
    Identifier(String),
    Boolean(bool),
    Plus,
    Minus,
    Asterisk,
    Slash,
    Dot,
}
```

To make things easy, we'll break tokenizing up into little functions which 
take some string slice (`&str`) and spit out either a token or an error.

```rust
fn tokenize_ident(src: &[u8]) -> Result<(Token, usize)> {
    let (got, bytes_read) = take_while(src, |ch| ch.is_alphanumeric())?;

    let tok = Token::Identifier(got.to_string());
    Ok((tok, bytes_read))
}
```

The `take_while()` function is just a helper which will call a closure on each
byte, continuing until the closure no longer returns `true`. 

It's pretty simple in that you just keep track of the current index, then 
afterwards convert everything from the start up to the index into a `&str`. 
Making sure to return the number of bytes consumed (that bit will be useful 
for later when we deal with spans).

```rust
fn take_while<F>(data: &[u8], mut pred: F) -> Result<(&str, usize)> 
where F: FnMut(char) -> bool
{
    let mut index = 0;

    while let Some(next) = data.get(index) {
        let next = *next as char;
        if pred(next) {
            index = index + 1;
        } else {
            break;
        }
    }

    if index == 0 {
        Err("No matches".into())
    } else {
        let got = str::from_utf8(&data[..index])?;
        Ok((got, index))
    }
}
```

Now lets make sure we can tokenize a normal identifier.

```rust
#[test]
fn tokenize_an_identifer() {
    let src = "Foo";
    let should_be = Token::Identifier(src.to_string());

    let (got, bytes_read) = tokenize_ident(src.as_bytes()).unwrap();

    assert_eq!(got, should_be);
    assert_eq!(bytes_read, src.len());
}
```