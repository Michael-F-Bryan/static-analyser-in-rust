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
    Asterisk,
    At, 
    Carat, 
    CloseParen, 
    CloseSquare, 
    Colon,
    Dot, 
    End,
    Equals,
    Minus, 
    OpenParen, 
    OpenSquare, 
    Plus,
    Semicolon,
    Slash,
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
    // identifiers can't start with a number
    match data.first() {
        Some(ch) => if (*ch as char).is_digit(10) {
            bail!("Identifiers can't start with a number")
        }
        None => bail!(ErrorKind::UnexpectedEOF)
    }

    let (got, bytes_read) = take_while(data, |ch| ch == '_' || ch.is_alphanumeric())?;

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
            assert_eq!(got, should_be, "Input was {:?}", src);
        }
    };
}
```

Now a test to check tokenizing identifiers becomes trivial.

```rust
lexer_test!(tokenize_a_single_letter, tokenize_ident, "F" => "F");
lexer_test!(tokenize_an_identifer, tokenize_ident, "Foo" => "Foo");
lexer_test!(tokenize_ident_containing_an_underscore, tokenize_ident, "Foo_bar" => "Foo_bar");
lexer_test!(FAIL: tokenize_ident_cant_start_with_number, tokenize_ident, "7Foo_bar");
lexer_test!(FAIL: tokenize_ident_cant_start_with_dot, tokenize_ident, ".Foo_bar");
```

Note that the macro calls `into()` on the result for us. Because we've defined
`From<&'a str>` for `Token`, we can use `"Foo"` as shorthand for the output.

It'also fairly easy to tokenize integers, they're just a continuous string of
digits. However if we also want to be able to deal with decimal numbers we
need to accept something that *may* look like two integers separated by a
dot. In this case we the predicate needs to keep track of how many `.`'s it
has seen, returning `false` the moment it sees more than one.


```rust
/// Tokenize a numeric literal.
fn tokenize_number(data: &[u8]) -> Result<(Token, usize)> {
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

    if seen_dot {
        let n: f64 = decimal.parse()?;
        Ok((Token::Decimal(n), bytes_read))
    } else {
        let n: usize = decimal.parse()?;
        Ok((Token::Integer(n), bytes_read))

    }
}
```

Something interesting with this approach is that a literal like `12.4.789` 
will be lexed as the decimal `12.4` followed by a `.789`, which is an invalid
float.


```rust
lexer_test!(tokenize_a_single_digit_integer, tokenize_number, "1" => 1);
lexer_test!(tokenize_a_longer_integer, tokenize_number, "1234567890" => 1234567890);
lexer_test!(tokenize_basic_decimal, tokenize_number, "12.3" => 12.3);
lexer_test!(tokenize_string_with_multiple_decimal_points, tokenize_number, "12.3.456" => 12.3);
lexer_test!(FAIL: cant_tokenize_a_string_as_a_decimal, tokenize_number, "asdfghj");
lexer_test!(tokenizing_decimal_stops_at_alpha, tokenize_number, "123.4asdfghj" => 123.4);
```

One last utility we're going to need is the ability to skip past whitespace 
characters and comments. These will be implemented as two separate functions
which are wrapped by a single `skip()`.

Let's deal with whitespace first seeing as that's easiest.

```rust
fn skip_whitespace(data: &[u8]) -> usize {
    match take_while(data, |ch| ch.is_whitespace()) {
        Ok((_, bytes_skipped)) => bytes_skipped,
        _ => 0,
    }
}

#[test]
fn skip_past_several_whitespace_chars() {
    let src = " \t\n\r";
    let should_be = src.len();

    let num_skipped = skip_whitespace(src.as_bytes());
    assert_eq!(num_skipped, should_be);
}

#[test]
fn skipping_whitespace_when_first_is_a_letter_returns_zero() {
    let src = "Hello World";
    let should_be = 0;

    let num_skipped = skip_whitespace(src.as_bytes());
    assert_eq!(num_skipped, should_be);
}
```

According to [the internets], a comment in Delphi can be written multiple ways.

> **Commenting Code**
> 
> Delphi uses `//` for a single line comment and both `{}` and `(**)` for 
> multiple line comments. Although you can nest different types of multiple 
> line comments, it is recommended that you don't.
> 
> **Compiler Directives - `$`**
>
> A special comment. Delphi compiler directives are in the form of
> `{$DIRECTIVE}`. Of interest for comments is using the `$IFDEF` compiler 
> directive to remark out code.

[the internets]: https://www.prestwoodboards.com/ASPSuite/KB/Document_View.asp?QID=101505


## The Main Tokenizer Function

To tie everything together, we'll use a method which matches the next
character against various patterns in turn. This is essentially just a big
`match` statement which defers to the small tokenizer functions we've built
up until now.


```rust
/// Try to lex a single token from the input stream.
pub fn tokenize(data: &[u8]) -> Result<(Token, usize)> {
    let next = match data.first() {
        Some(c) => *c as char,
        None => bail!(ErrorKind::UnexpectedEOF),
    };

    match next {
        '.' => Ok((Token::Dot, 1)),
        '+' => Ok((Token::Plus, 1)),
        '-' => Ok((Token::Minus, 1)),
        '*' => Ok((Token::Asterisk, 1)),
        '/' => Ok((Token::Slash, 1)),
        '@' => Ok((Token::At, 1)),
        '^' => Ok((Token::Carat, 1)),
        '(' => Ok((Token::OpenParen, 1)),
        ')' => Ok((Token::CloseParen, 1)),
        '[' => Ok((Token::OpenSquare, 1)),
        ']' => Ok((Token::CloseSquare, 1)),
        '0' ... '9' => tokenize_number(data),
        c @ '_' | c if c.is_alphabetic() => tokenize_ident(data),
        other => bail!(ErrorKind::UnknownCharacter(other)),
    }
}
```

Now lets test it, in theory we should get identical results to the other tests
written up til now.

```rust
lexer_test!(central_tokenizer_ident, tokenize, "hello" => "hello");
lexer_test!(central_tokenizer_integer, tokenize, "1234" => 1234);
lexer_test!(central_tokenizer_decimal, tokenize, "123.4" => 123.4);
```