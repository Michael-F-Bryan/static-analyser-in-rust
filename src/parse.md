# Parsing

```rust
//! Parse a stream of `Tokens` into an *Abstract Syntax Tree* we can use for
//! the later steps.
```

Now that we've turned the source code into tokens we can construct a more
computer-friendly representation for the program. This representation is
often called an *Abstract Syntax Tree* because it's a high-level tree
datastructure which reflects a program's syntax.

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
