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
use lex::{Token, TokenKind};
use codemap::{Span, FileMap};
use parse::ast::{Literal, LiteralKind};
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
  fn parse_literal(&mut self) -> Option<Literal> {
    match self.peek() {
      Some(&TokenKind::Integer(_)) | 
      Some(&TokenKind::Decimal(_)) | 
      Some(&TokenKind::QuotedString(_)) => {},
      _ => return None,
    };

    let next = self.next().expect("unreachable");
    let lit_kind = match next.kind {
      TokenKind::Integer(i) => LiteralKind::Integer(i),
      TokenKind::Decimal(d) => LiteralKind::Decimal(d),
      TokenKind::QuotedString(ref s) => LiteralKind::String(s.clone()),
      ref other => panic!("Unreachable token kind: {:?}", other),
    };

    Some(Literal {
      span: next.span,
      kind: lit_kind
    })
  }
}
```