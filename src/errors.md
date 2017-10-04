# Error Handling

This is just some code to hook `error-chain` up so we can use it for internal
errors. Feel free to skip past this.


```rust
//! Types and traits used for internal errors.

error_chain!{
    errors {
        UnexpectedEOF {
            display("Unexpected EOF")
            description("Unexpected EOF")
        }

        UnknownCharacter(ch: char) {
            display("Unknown Character, {:?}", ch)
            display("Unknown Character")
        }
    }

    foreign_links {
        Io(::std::io::Error) #[doc = "Wrapper around a `std::io::Error`"];
        Utf8(::std::str::Utf8Error) #[doc = "An error parsing data as UTF-8"];
        FloatParsing(::std::num::ParseFloatError) #[doc = "A float parsing error"];
        IntParsing(::std::num::ParseIntError) #[doc = "An integer parsing error"];
        
    }
}
```