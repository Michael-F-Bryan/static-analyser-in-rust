# Lexical Analysis

It's always nice to add doc-comments so rustdoc knows what this module does.

```rust
//! Module for performing lexical analysis on source code.
```

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
/// Any valid token in the Delphi programming language.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(usize),
    Decimal(f64),
    Identifier(String),
    QuotedString(String),
    End,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Dot,
    Colon,
    Semicolon,
    Equals,
}
```

We'll also want to implement some helpers to make conversion more ergonomic.

```rust
impl From<String> for Token {
    fn from(other: String) -> Token {
        Token::Identifier(other)
    }
}

impl<'a> From<&'a str> for Token {
    fn from(other: &'a str) -> Token {
        Token::Identifier(other.to_string())
    }
}

impl From<usize> for Token {
    fn from(other: usize) -> Token {
        Token::Integer(other)
    }
}

impl From<f64> for Token {
    fn from(other: f64) -> Token {
        Token::Decimal(other)
    }
}
```


## Tokenizing Individual Atoms

To make things easy, we'll break tokenizing up into little functions which 
take some string slice (`&str`) and spit out either a token or an error.

```rust
fn tokenize_ident(data: &[u8]) -> Result<(Token, usize)> {
    let (got, bytes_read) = take_while(data, |ch| ch.is_alphanumeric())?;

    let tok = Token::Identifier(got.to_string());
    Ok((tok, bytes_read))
}
```

As a general rule, our tokenizer functions will look like this

```rust
type Tokenizer<T> = fn(&[u8]) -> Result<(T, usize)>;
```

The `take_while()` function is just a helper which will call a closure on each
byte, continuing until the closure no longer returns `true`. 

It's pretty simple in that you just keep track of the current index, then 
afterwards convert everything from the start up to the index into a `&str`. 
Making sure to return the number of bytes consumed (that bit will be useful 
for later when we deal with spans).

```rust
/// Consumes bytes while a predicate evaluates to true.
fn take_while_raw<F>(data: &[u8], mut pred: F) -> Result<(&[u8], usize)>  
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
        Ok((&data[..index], index))
    }
}

/// Consumes bytes while a predicate evaluates to true, then converts them
/// to a string.
fn take_while<F>(data: &[u8], pred: F) -> Result<(&str, usize)> 
where F: FnMut(char) -> bool
{
    take_while_raw(data, pred).and_then(|(bytes, n)| {
        let as_str = str::from_utf8(bytes)?;
        Ok((as_str, n))
    })
}
```

Now lets test it! To make life easier, we'll create a helper macro which 
generates a test for us. We just need to pass in a test name and the function
being tested, and an input string and expected output. Then the macro will do
the rest.


```rust
macro_rules! lexer_test {
    (FAIL: $name:ident, $func:ident, $src:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let src: &str = $src;
            let func: Tokenizer<_> = $func;

            let got = func(src.as_bytes());
            assert!(got.is_err(), "{:?} should be an error", got);
        }
    };
    ($name:ident, $func:ident, $src:expr => $should_be:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let src: &str = $src;
            let should_be = Token::from($should_be);
            let func: Tokenizer<_> = $func;

            let (got, _bytes_read) = func(src.as_bytes()).unwrap();
            assert_eq!(got, should_be);
        }
    };
}
```

Now a test to check tokenizing identifiers becomes trivial.

```rust
lexer_test!(tokenize_a_single_letter, tokenize_ident, "F" => "F");
lexer_test!(tokenize_an_identifer, tokenize_ident, "Foo" => "Foo");
```

Note that the macro calls `into()` on the result for us. Because we've defined
`From<&'a str>` for `Token`, we can use `"Foo"` as shorthand for the output.

It's also fairly easy to tokenize integers. They're just a continuous string
of digits.

```rust
fn tokenize_integer(data: &[u8]) -> Result<(Token, usize)> {
    let (integer, bytes_read) = take_while(data, |c| c.is_digit(10))?;
    let integer = integer.parse().expect("Already checked this is a number");

    Ok((Token::Integer(integer), bytes_read))
}
```

And to test it:

```rust
lexer_test!(tokenize_a_single_digit_integer, tokenize_integer, "1" => 1);
lexer_test!(tokenize_a_longer_integer, tokenize_integer, "1234567890" => 1234567890);
lexer_test!(tokenizing_integers_consumes_only_up_to_dot, tokenize_integer, "12.34" => 12);
```

Lexing a decimal number is *almost* as easy as integers. In this case we the
predicate needs to keep track of how many `.`'s it has seen, returning `false`
the moment it sees more than one.


```rust
fn tokenize_decimal(data: &[u8]) -> Result<(Token, usize)> {
    let mut seen_dot = false;

    let (decimal, bytes_read) = take_while(data, |c| {
        if c.is_digit(10) {
            true
        } else if c == '.' {
            if !seen_dot {
                seen_dot = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    })?;

    let n: f64 = decimal.parse()?;
    Ok((Token::Decimal(n), bytes_read))
}
```

Something interesting with this approach is that a literal like `12.4.789` 
will be lexed as the decimal `12.4` followed by a `.789`, which is an invalid
float.


```rust
lexer_test!(lex_integer_as_float, tokenize_decimal, "123" => 123.0);
lexer_test!(tokenize_basic_decimal, tokenize_decimal, "12.3" => 12.3);
lexer_test!(tokenize_string_with_multiple_decimal_points, tokenize_decimal, "12.3.456" => 12.3);
lexer_test!(FAIL: cant_tokenize_a_string_as_a_decimal, tokenize_decimal, "asdfghj");
lexer_test!(tokenizing_decimal_stops_at_alpha, tokenize_decimal, "123.4asdfghj" => 123.4);
```