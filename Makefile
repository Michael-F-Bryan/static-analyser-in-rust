OPEN := xdg-open

usage:
	-@ echo "Usage:"
	-@ echo "    build        Build both the crate and accompanying book"
	-@ echo "    word-count   Get some (rough) statistics about the repository"
	-@ echo "    usage        Print this help text"

open: build
	cargo doc --open
	$OPEN ./book/index.html

build: build-crate build-docs

build-crate:
	cargo build

build-docs:
	mdbook build


word-count:
	-@ wc --words $$(find src/ -name "*.md")

