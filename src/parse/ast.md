# The Abstract Syntax Tree

```rust
#![allow(missing_docs)]
use codemap::Span;
```

The most basic element of the AST is a `Literal`. These are either integer,
float, or string literals.

```rust
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum LiteralKind {
    Integer(usize),
    Decimal(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub span: Span,
    pub kind: LiteralKind,
}
```