# Error Handling

This is just some code to hook `error-chain` up so we can use it for internal
errors. Feel free to skip past this.


```rust
error_chain!{
    foreign_links {
        Io(::std::io::Error) #[doc = "Wrapper around a `std::io::Error`"];
        Utf8(::std::str::Utf8Error) #[doc = "An error parsing data as UTF-8"];
    }
}
```