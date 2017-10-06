# Parsing


Now that we've turned the source code into tokens we can construct a more
computer-friendly representation for the program. This representation is
often called an *Abstract Syntax Tree* because it's a high-level tree
datastructure which reflects a program's syntax.


## The General Idea

Before we start with parsing, lets look at an example chunk of Delphi code
to get a feel for the language. A *unit file* is the basic building block of a
Delphi program, analogous to a `*.c` file. The `Main()` function is typically
elsewhere in GUI programs because an application's endpoint is typically 
managed by the GUI framework or IDE.


```delphi
unit Unit1;

interface

uses
  Windows, Messages, SysUtils, Variants, Classes, Graphics, Controls, Forms,
  Dialogs, StdCtrls;

type
  TForm1 = class(TForm)
    Label1: TLabel;      // The label we have added
    Button1: TButton;    // The button we have added
    procedure Button1Click(Sender: TObject);
  private
    { private declarations }
  public
    { public declarations }
  end;

var
  Form1: TForm1;

implementation

{$R *.dfm}

// The button action we have added
procedure TForm1.Button1Click(Sender: TObject);
begin
  Label1.Caption := 'Hello World';    // Label changed when button pressed
end;

end.
```

At a very high level, a unit file consists of a `unit` statement declaring the 
unit's name, followed by the `interface` (kinda like a `*.h` file) then an
`implementation` section, before ending with a `end.`.

There's a formal language used to express a language's grammar called 
*Backus–Naur form*. That previous paragraph would translate to something like 
the following:

```ebnf
file        = unit_decl interface implementation "end.";
unit_decl   = "unit" unit_name SEMICOLON;
unit_name   = WORD;
interface   = "interface" uses types vars;
uses        = "uses" WORD ("," WORD)* SEMICOLON
             | ε;
types       = "type" type_decl*;
vars        = "var" var_decl*;
```

With the terminals (`WORD`, `SEMICOLON`, and friends) being their usual selves.

Delphi has a pretty simple syntax, so we're going to use a standard recursive
descent parser. This is just an object which has a method roughly corresponding 
to each rule in the language's grammar.


## The Parser Object

As usual, before we can do anything else we're going to have to import a couple
dependencies.

```rust
use std::rc::Rc;
use std::borrow::Borrow;

use lex::{Token, TokenKind};
use codemap::{Span, FileMap};
use parse::ast::{Literal, LiteralKind, Ident, DottedIdent};
use errors::*;
```

The `Parser` itself just contains the tokens and their corresponding `FileMap`.

```rust
/// A parser for turning a stream of tokens into a Abstract Syntax Tree.
#[derive(Debug)]
pub struct Parser {
  tokens: Vec<Token>,
  filemap: Rc<FileMap>,
  current_index: usize,
}

impl Parser {
  /// Create a new parser.
  pub fn new(tokens: Vec<Token>, filemap: Rc<FileMap>) -> Parser {
    let current_index = 0;
    Parser { tokens, filemap, current_index }
  }

  /// Peek at the current token.
  fn peek(&self) -> Option<&TokenKind> {
    self.tokens.get(self.current_index).map(|t| &t.kind)
  }

  /// Get the current token, moving the index along one.
  fn next(&mut self) -> Option<&Token> {
    let tok = self.tokens.get(self.current_index);

    if tok.is_some() {
      self.current_index += 1;
    }

    tok
  }
}
```

We'll implement the various grammar rules from the bottom up. Meaning we'll 
start with the very basics like expressions, then build things up until we
get to the overall program.

First up lets have a go at parsing `Literals`. We do it in two steps, first
you peek at the next token to make sure it's a kind you expect, then you
unpack the token and convert it into it's equivalent AST node. A lot of the
pattern matching boilerplate can be minimised with the judicious use of macros.

```rust
impl Parser {
  fn parse_literal(&mut self) -> Result<Literal> {
    match self.peek() {
      Some(&TokenKind::Integer(_)) | 
      Some(&TokenKind::Decimal(_)) | 
      Some(&TokenKind::QuotedString(_)) => {},
      Some(_) => bail!("Expected a literal"),
      None => bail!(ErrorKind::UnexpectedEOF),
    };

    let next = self.next().expect("unreachable");
    let lit_kind = match next.kind {
      TokenKind::Integer(i) => LiteralKind::Integer(i),
      TokenKind::Decimal(d) => LiteralKind::Decimal(d),
      TokenKind::QuotedString(ref s) => LiteralKind::String(s.clone()),
      ref other => panic!("Unreachable token kind: {:?}", other),
    };

    Ok(Literal {
      span: next.span,
      kind: lit_kind
    })
  }
}
```

Like the tokenizing module, we're going to need to write lots of tests to 
check our parser recognises things as we expect them to. Unfortunately the
types and syntactic structures used will be slightly different, so we'll
use macros to abstract away a lot of the boilerplate.

```rust
macro_rules! parser_test {
  ($name:ident, $method:ident, $src:expr => $should_be:expr) =>  {
    #[cfg(test)]
    #[test]
    fn $name() {
      // create a codemap and tokenize our input string
      let mut codemap = $crate::codemap::CodeMap::new();
      let filemap = codemap.insert_file("dummy.pas", $src);
      let tokenized = $crate::lex::tokenize(filemap.contents())
        .chain_err(|| "Tokenizing failed")
        .unwrap();
      let tokens = filemap.register_tokens(tokenized);

      let should_be = $should_be;

      let mut parser = Parser::new(tokens, filemap);
      let got = parser.$method().unwrap();

      assert_eq!(got, should_be);
    }
  }
}
```

Now we have our basic test harness set up, lets see if literal parsing works.

```rust
parser_test!(integer_literal, parse_literal, "123" => LiteralKind::Integer(123));
parser_test!(parse_float_literal, parse_literal, "12.3" => LiteralKind::Decimal(12.3));
// TODO: re-enable this when string parsing is implemented
// parser_test!(parse_string_literal, parse_literal, "'12.3'" => LiteralKind::String("12.3".to_string()));
```

Another easy thing to implement is parsing identifiers and dotted identifiers 
(e.g. `foo.bar.baz`).


```rust
impl Parser {
  fn parse_ident(&mut self) -> Result<Ident> {
    match self.peek() {
      Some(&TokenKind::Identifier(_)) => {},
      _ => bail!("Expected an identifier"),
    }

    let next = self.next().unwrap();

    if let TokenKind::Identifier(ref ident) = next.kind {
      Ok(Ident {
        span: next.span,
        name: ident.clone(),
      })
    } else {
      unreachable!()
    }
  }

  fn parse_dotted_ident(&mut self) -> Result<DottedIdent> {
    let first = self.parse_ident()?;
    let mut parts = vec![first];

    while self.peek() == Some(&TokenKind::Dot) {
      let _ = self.next();
      let next = self.parse_ident()?;
      parts.push(next);
    }

    let span = parts.iter().skip(1).fold(parts[0].span, |l, r| self.filemap.merge(l, r.span));

    Ok(DottedIdent { span, parts })    
  }
}
```

We also want to test these.

```rust
parser_test!(parse_a_basic_ident, parse_ident, "foo" => "foo");
parser_test!(parse_a_dotted_ident, parse_dotted_ident, "foo.bar.baz" => ["foo", "bar", "baz"].borrow());
```