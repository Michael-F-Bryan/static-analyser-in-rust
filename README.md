# Create a Static Analyser in Rust

[![Build Status](https://travis-ci.org/Michael-F-Bryan/static-analyser-in-rust.svg?branch=master)](https://travis-ci.org/Michael-F-Bryan/static-analyser-in-rust)

An experiment in using *literate programming* to write a static analysis tool
for Delphi code. Feel free to read along if you want.


## Building

If you want to build and read this locally you'll need to have the following 
installed:

- Rust (via [rustup])
- tango (`cargo install tango`)
- mdbook (`cargo install mdbook`)

If you've freshly cloned the repo then `src/lib.rs` won't yet exist. Cargo 
doesn't particularly like this, so we need to manually run `tango` to generate
the Rust code.

```
$ tango
```

If you look at the `src/` directory you'll see two copies of everything, one in
markdown (`*.md`) and the other in Rust (`*.rs`). If there are ever any compile
errors, it's often super useful to be able to look at the actual source code 
being compiled.

If you want to look at the book version of the code, you'll need to run 
`mdbook`.

```
$ mdbook build --open
```

And the `rustdoc` documentation can be viewed the usual way.

```
$ cargo doc --open
```

[rustup]: https://rustup.rs/