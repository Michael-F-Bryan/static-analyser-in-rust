//@ # Lexical Analysis
//@
//@ It's always nice to add doc-comments so rustdoc knows what this module does.

//! Module for performing lexical analysis on source code.

//@ Before anything else, lets import some things we'll require.

use std::str;
use codemap::Span;
use errors::*;

//@ A lexer's job is to turn normal strings (which a human can read) into
//@ something more computer-friendly called a `Token`. In this crate, a `Token`
//@ will be comprised of a `Span` (more about that [later]), and a `TokenKind`
//@ which lets us know which type of token we are dealing with. A `TokenKind` can
//@ be multiple different types representing multiple different things, so it
//@ makes sense to use a Rust enum here.
//@
//@ [later]: ./codemap.html

/// Any valid token in the Delphi programming language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(tag = "type")]
pub enum TokenKind {
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

//@ We'll also want to implement some helpers to make conversion more ergonomic.

impl From<String> for TokenKind {
    fn from(other: String) -> TokenKind {
        TokenKind::Identifier(other)
    }
}

impl<'a> From<&'a str> for TokenKind {
    fn from(other: &'a str) -> TokenKind {
        TokenKind::Identifier(other.to_string())
    }
}

impl From<usize> for TokenKind {
    fn from(other: usize) -> TokenKind {
        TokenKind::Integer(other)
    }
}

impl From<f64> for TokenKind {
    fn from(other: f64) -> TokenKind {
        TokenKind::Decimal(other)
    }
}


//@ ## Tokenizing Individual Atoms
//@
//@ To make things easy, we'll break tokenizing up into little functions which 
//@ take some string slice (`&str`) and spit out either a token or an error.

fn tokenize_ident(data: &str) -> Result<(TokenKind, usize)> {
    // identifiers can't start with a number
    match data.chars().next() {
        Some(ch) if ch.is_digit(10) => bail!("Identifiers can't start with a number"),
        None => bail!(ErrorKind::UnexpectedEOF),
        _ => {},
    }

    let (got, bytes_read) = take_while(data, |ch| ch == '_' || ch.is_alphanumeric())?;

    // TODO: Recognise keywords using a `match` statement here.

    let tok = TokenKind::Identifier(got.to_string());
    Ok((tok, bytes_read))
}

//@ The `take_while()` function is just a helper which will call a closure on each
//@ byte, continuing until the closure no longer returns `true`. 
//@
//@ It's pretty simple in that you just keep track of the current index, then 
//@ afterwards convert everything from the start up to the index into a `&str`. 
//@ Making sure to return the number of bytes consumed (that bit will be useful 
//@ for later when we deal with spans).

/// Consumes bytes while a predicate evaluates to true.
fn take_while<F>(data: &str, mut pred: F) -> Result<(&str, usize)>  
where F: FnMut(char) -> bool
{
    let mut current_index = 0;

    for ch in data.chars() {
        let should_continue = pred(ch);

        if !should_continue {
            break;
        }

        current_index += ch.len_utf8();
    }

    if current_index == 0 {
        Err("No Matches".into())
    } else {
        Ok((&data[..current_index], current_index))
    }
}

//@ Now lets test it! To make life easier, we'll create a helper macro which 
//@ generates a test for us. We just need to pass in a test name and the function
//@ being tested, and an input string and expected output. Then the macro will do
//@ the rest.


macro_rules! lexer_test {
    (FAIL: $name:ident, $func:ident, $src:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let src: &str = $src;
            let func = $func;

            let got = func(src);
            assert!(got.is_err(), "{:?} should be an error", got);
        }
    };
    ($name:ident, $func:ident, $src:expr => $should_be:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let src: &str = $src;
            let should_be = TokenKind::from($should_be);
            let func = $func;

            let (got, _bytes_read) = func(src).unwrap();
            assert_eq!(got, should_be, "Input was {:?}", src);
        }
    };
}

//@ Now a test to check tokenizing identifiers becomes trivial.

lexer_test!(tokenize_a_single_letter, tokenize_ident, "F" => "F");
lexer_test!(tokenize_an_identifer, tokenize_ident, "Foo" => "Foo");
lexer_test!(tokenize_ident_containing_an_underscore, tokenize_ident, "Foo_bar" => "Foo_bar");
lexer_test!(FAIL: tokenize_ident_cant_start_with_number, tokenize_ident, "7Foo_bar");
lexer_test!(FAIL: tokenize_ident_cant_start_with_dot, tokenize_ident, ".Foo_bar");

//@ Note that the macro calls `into()` on the result for us. Because we've defined
//@ `From<&'a str>` for `TokenKind`, we can use `"Foo"` as shorthand for the output.
//@
//@ It'also fairly easy to tokenize integers, they're just a continuous string of
//@ digits. However if we also want to be able to deal with decimal numbers we
//@ need to accept something that *may* look like two integers separated by a
//@ dot. In this case we the predicate needs to keep track of how many `.`'s it
//@ has seen, returning `false` the moment it sees more than one.


/// Tokenize a numeric literal.
fn tokenize_number(data: &str) -> Result<(TokenKind, usize)> {
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
        Ok((TokenKind::Decimal(n), bytes_read))
    } else {
        let n: usize = decimal.parse()?;
        Ok((TokenKind::Integer(n), bytes_read))

    }
}

//@ Something interesting with this approach is that a literal like `12.4.789` 
//@ will be lexed as the decimal `12.4` followed by a `.789`, which is an invalid
//@ float.


lexer_test!(tokenize_a_single_digit_integer, tokenize_number, "1" => 1);
lexer_test!(tokenize_a_longer_integer, tokenize_number, "1234567890" => 1234567890);
lexer_test!(tokenize_basic_decimal, tokenize_number, "12.3" => 12.3);
lexer_test!(tokenize_string_with_multiple_decimal_points, tokenize_number, "12.3.456" => 12.3);
lexer_test!(FAIL: cant_tokenize_a_string_as_a_decimal, tokenize_number, "asdfghj");
lexer_test!(tokenizing_decimal_stops_at_alpha, tokenize_number, "123.4asdfghj" => 123.4);

//@ One last utility we're going to need is the ability to skip past whitespace 
//@ characters and comments. These will be implemented as two separate functions
//@ which are wrapped by a single `skip()`.
//@
//@ Let's deal with whitespace first seeing as that's easiest.

fn skip_whitespace(data: &str) -> usize {
    match take_while(data, |ch| ch.is_whitespace()) {
        Ok((_, bytes_skipped)) => bytes_skipped,
        _ => 0,
    }
}

#[test]
fn skip_past_several_whitespace_chars() {
    let src = " \t\n\r123";
    let should_be = 4;

    let num_skipped = skip_whitespace(src);
    assert_eq!(num_skipped, should_be);
}

#[test]
fn skipping_whitespace_when_first_is_a_letter_returns_zero() {
    let src = "Hello World";
    let should_be = 0;

    let num_skipped = skip_whitespace(src);
    assert_eq!(num_skipped, should_be);
}

//@ > **TODO:** Tokenize string literals
//@
//@ According to [the internets], a comment in Delphi can be written multiple ways.
//@
//@ > **Commenting Code**
//@ > 
//@ > Delphi uses `//` for a single line comment and both `{}` and `(**)` for 
//@ > multiple line comments. Although you can nest different types of multiple 
//@ > line comments, it is recommended that you don't.
//@ > 
//@ > **Compiler Directives - `$`**
//@ >
//@ > A special comment. Delphi compiler directives are in the form of
//@ > `{$DIRECTIVE}`. Of interest for comments is using the `$IFDEF` compiler 
//@ > directive to remark out code.
//@
//@ [the internets]: https://wwgetw.prestwoodboards.com/ASPSuite/KB/Document_View.asp?QID=101505

fn skip_comments(src: &str) -> usize {
    let pairs = [("//", "\n"), ("{", "}"), ("(*", "*)")];

    for &(pattern, matcher) in &pairs {
        if src.starts_with(pattern) {
            let leftovers = skip_until(src, matcher);
            return src.len() - leftovers.len();
        }
    }

    0
}

fn skip_until<'a>(mut src: &'a str, pattern: &str) -> &'a str {
    while !src.is_empty() && !src.starts_with(pattern) {
        let next_char_size = src.chars().next().expect("The string isn't empty").len_utf8();
        src = &src[next_char_size..];
    }

    &src[pattern.len()..]
}

macro_rules! comment_test {
    ($name:ident, $src:expr => $should_be:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let got = skip_comments($src);
            assert_eq!(got, $should_be);
        }
    }
}

comment_test!(slash_slash_skips_to_end_of_line, "// foo bar { baz }\n 1234" => 19);
comment_test!(comment_skip_curly_braces, "{ baz \n 1234} hello wor\nld" => 13);
comment_test!(comment_skip_round_brackets, "(* Hello World *) asd" => 17);
comment_test!(comment_skip_ignores_alphanumeric, "123 hello world" => 0);
comment_test!(comment_skip_ignores_whitespace, "   (* *) 123 hello world" => 0);

//@ Lastly, we group the whitespace and comment skipping together seeing as they
//@ both do the job of skipping characters we don't care about.

/// Skip past any whitespace characters or comments.
fn skip(src: &str) -> usize {
    let mut remaining = src;

    loop {
        let ws = skip_whitespace(remaining);
        remaining = &remaining[ws..];
        let comments = skip_comments(remaining);
        remaining = &remaining[comments..];

        if ws + comments == 0 {
            return src.len() - remaining.len();
        }
    }
}


//@ ## The Main Tokenizer Function
//@
//@ To tie everything together, we'll use a method which matches the next
//@ character against various patterns in turn. This is essentially just a big
//@ `match` statement which defers to the small tokenizer functions we've built
//@ up until now.


/// Try to lex a single token from the input stream.
pub fn tokenize_single_token(data: &str) -> Result<(TokenKind, usize)> {
    let next = match data.chars().next() {
        Some(c) => c,
        None => bail!(ErrorKind::UnexpectedEOF),
    };

    let (tok, length) = match next {
        '.' => (TokenKind::Dot, 1),
        '=' => (TokenKind::Equals, 1),
        '+' => (TokenKind::Plus, 1),
        '-' => (TokenKind::Minus, 1),
        '*' => (TokenKind::Asterisk, 1),
        '/' => (TokenKind::Slash, 1),
        '@' => (TokenKind::At, 1),
        '^' => (TokenKind::Carat, 1),
        '(' => (TokenKind::OpenParen, 1),
        ')' => (TokenKind::CloseParen, 1),
        '[' => (TokenKind::OpenSquare, 1),
        ']' => (TokenKind::CloseSquare, 1),
        '0' ... '9' => tokenize_number(data).chain_err(|| "Couldn't tokenize a number")?,
        c @ '_' | c if c.is_alphabetic() => tokenize_ident(data)
            .chain_err(|| "Couldn't tokenize an identifier")?,
        other => bail!(ErrorKind::UnknownCharacter(other)),
    };

    Ok((tok, length))
}

//@ Now lets test it, in theory we should get identical results to the other tests
//@ written up til now.

//@@ lexer_test!(central_tokenizer_ident, tokenize_single_token, "hello" => "hello");
lexer_test!(central_tokenizer_integer, tokenize_single_token, "1234" => 1234);
lexer_test!(central_tokenizer_decimal, tokenize_single_token, "123.4" => 123.4);
lexer_test!(central_tokenizer_dot, tokenize_single_token, "." => TokenKind::Dot);
lexer_test!(central_tokenizer_plus, tokenize_single_token, "+" => TokenKind::Plus);
lexer_test!(central_tokenizer_minus, tokenize_single_token, "-" => TokenKind::Minus);
lexer_test!(central_tokenizer_asterisk, tokenize_single_token, "*" => TokenKind::Asterisk);
lexer_test!(central_tokenizer_slash, tokenize_single_token, "/" => TokenKind::Slash);
lexer_test!(central_tokenizer_at, tokenize_single_token, "@" => TokenKind::At);
lexer_test!(central_tokenizer_carat, tokenize_single_token, "^" => TokenKind::Carat);
lexer_test!(central_tokenizer_equals, tokenize_single_token, "=" => TokenKind::Equals);
lexer_test!(central_tokenizer_open_paren, tokenize_single_token, "(" => TokenKind::OpenParen);
lexer_test!(central_tokenizer_close_paren, tokenize_single_token, ")" => TokenKind::CloseParen);
lexer_test!(central_tokenizer_open_square, tokenize_single_token, "[" => TokenKind::OpenSquare);
lexer_test!(central_tokenizer_close_square, tokenize_single_token, "]" => TokenKind::CloseSquare);


//@ ## Tying It All Together
//@
//@ Now we can write the overall tokenizer function. However, because this process
//@ involves a lot of state, it'll be easier to encapsulate everything in its own
//@ type while still exposing a high-level `tokenize()` function to users.


struct Tokenizer<'a> {
    current_index: usize,
    remaining_text: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &str) -> Tokenizer {
        Tokenizer {
            current_index: 0,
            remaining_text: src,
        }
    }

    fn next_token(&mut self) -> Result<Option<(TokenKind, usize, usize)>> {
        self.skip_whitespace();

        if self.remaining_text.is_empty() {
            Ok(None)
        } else {
            let start = self.current_index;
            let tok = self._next_token()
                .chain_err(|| ErrorKind::MessageWithLocation(self.current_index,
                    "Couldn't read the next token"))?;
            let end = self.current_index;
            Ok(Some((tok, start, end)))
        }
    }

    fn skip_whitespace(&mut self) {
        let skipped = skip(self.remaining_text);
        self.chomp(skipped);
    }

    fn _next_token(&mut self) -> Result<TokenKind> {
        let (tok, bytes_read) = tokenize_single_token(self.remaining_text)?;
        self.chomp(bytes_read);

        Ok(tok)
    }

    fn chomp(&mut self, num_bytes: usize) {
        self.remaining_text = &self.remaining_text[num_bytes..];
        self.current_index += num_bytes;
    }
}

/// Turn a string of valid Delphi code into a list of tokens, including the 
/// location of that token's start and end point in the original source code.
///
/// Note the token indices represent the half-open interval `[start, end)`, 
/// equivalent to `start .. end` in Rust.
pub fn tokenize(src: &str) -> Result<Vec<(TokenKind, usize, usize)>> {
    let mut tokenizer = Tokenizer::new(src);
    let mut tokens = Vec::new();

    while let Some(tok) = tokenizer.next_token()? {
        tokens.push(tok);
    }

    Ok(tokens)
}

//@ Because we also want to make sure the location of tokens are correct, testing 
//@ this will be a little more involved. We essentially need to write up some
//@ (valid) Delphi code, manually inspect it, then make sure we get back *exactly*
//@ what we expect. Byte indices and all.

#[cfg(test)]
#[test]
fn tokenize_a_basic_expression() {
    let src = "foo = 1 + 2.34";
    let should_be = vec![
        (TokenKind::from("foo"), 0, 3),
        (TokenKind::Equals, 4, 5),
        (TokenKind::from(1), 6, 7),
        (TokenKind::Plus, 8, 9),
        (TokenKind::from(2.34), 10, 14),
    ];

    let got = tokenize(src).unwrap();
    assert_eq!(got, should_be);
}

#[cfg(test)]
#[test]
fn tokenizer_detects_invalid_stuff() {
    let src = "foo bar `%^&\\";
    let index_of_backtick = 8;

    let err = tokenize(src).unwrap_err();
    match err.kind() {
        &ErrorKind::MessageWithLocation(loc, _) => assert_eq!(loc, index_of_backtick),
        other => panic!("Unexpected error: {}", other),
    }
}

//@ You'll probably notice that we're returning a `TokenKind` and a pair of integers
//@ inside a tuple, which isn't overly idiomatic. Idiomatic Rust would bundle 
//@ these up into a more strongly typed tuple of `TokenKind` and `Span`, where a span
//@ corresponds to the start and end indices of the token. 
//@
//@ The reason we do things slightly strangly is that we're using a `CodeMap` to
//@ manage all these `Span`s, so when the caller calls the `tokenize()` function
//@ it's their responsibility to insert these token locations into a `CodeMap`.
//@ By returning a plain tuple of integers it means we can defer dealing with the
//@ `CodeMap` until later on. Vastly simplifying the tokenizing code.
//@
//@ For completeness though, here is the `Token` people will be using. We haven't
//@ created any in this module, but it makes sense for its definition to be here.

/// A valid Delphi source code token.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// The token's location relative to the rest of the files being 
    /// processed.
    pub span: Span,
    /// What kind of token is this?
    pub kind: TokenKind,
}

impl Token {
    /// Create a new token out of a `Span` and something which can be turned 
    /// into a `TokenKind`.
    pub fn new<K: Into<TokenKind>>(span: Span, kind: K) -> Token {
        let kind = kind.into();
        Token { span, kind }
    }
}

impl<T> From<T> for Token 
where T: Into<TokenKind> {
    fn from(other: T) -> Token {
        Token::new(Span::dummy(), other)
    }
}

//@ And that's about it for lexical analysis. We've now got the basic building 
//@ blocks of a compiler/static analyser, and are able to move onto the next 
//@ step... Actually making sense out of all these tokens!
