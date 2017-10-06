//@ # Macros
//@
//@ To make testing easier, we're going to create a `tok!()` macro which uses 
//@ the magic of `Into<Token>` to intelligently create tokens.

/// Shorthand macro for generating a token from *anything* which can be 
/// converted into a `TokenKind`, or any of the `TokenKind` variants.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate static_analyser;
///
/// # fn main() {
/// tok!(Dot);
/// tok!(123);
/// tok!(3.14);
/// tok!(OpenParen);
/// # }
/// ```
#[macro_export]
macro_rules! tok {
    ($thing:tt) =>  {
        {
            #[allow(unused_imports)]
            use $crate::lex::TokenKind::*;
            $crate::lex::Token::from($thing)
        }
    };
}

//@ We also want to add some tests to make sure the code it expands to is sane.
//@ Amusingly enough, we're going to write a macro to help generate tests to
//@ test our `tok!()` macro. This helps sidestep the otherwise unnecessary 
//@ boilerplate around creating loads of *almost similar* tests which may deal
//@ with different types and syntactic structures.

#[cfg(test)]
mod tests {
    use codemap::Span;
    use lex::{Token, TokenKind};

    macro_rules! token_macro_test {
        ($name:ident, $from:tt => $to:expr) => {
            #[test]
            fn $name() {
                let got: Token = tok!($from);
                let should_be = Token::new(Span::dummy(), $to);

                assert_eq!(got, should_be);
            }
        }
    }

    token_macro_test!(tok_expands_to_dot, Dot => TokenKind::Dot);
    token_macro_test!(tok_expands_to_openparen, OpenParen => TokenKind::OpenParen);
    token_macro_test!(tok_expands_to_integer, 1234 => TokenKind::Integer(1234));
    token_macro_test!(tok_expands_to_decimal, 12.34 => TokenKind::Decimal(12.34));
    token_macro_test!(tok_expands_to_identifier, "Hello" => TokenKind::Identifier("Hello".to_string()));
}

//@ When used this way, macros have a tendency to give horrible error messages.
//@ To make sure this won't happen I tried to pass in a non-existent `TokenKind`
//@ variant and see what happens:
//@
//@ ```
//@ error[E0425]: cannot find value `DoesntExist` in this scope
//@ --> src/parse/macros.rs:41:55
//@ |
//@ 41 |     token_macro_test!(use_tok_with_nonexistent_thing, DoesntExist => TokenKind::Dot);
//@ |                                                          ^^^^^^^^^^^ not found in this scope
//@ ```
//@
//@ I'm fairly happy with the results.
