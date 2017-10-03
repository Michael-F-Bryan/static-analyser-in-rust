OPEN := xdg-open


usage:
	-@ echo "Usage:"
	-@ echo "    open         Build the project and open the tutorial in your browser"
	-@ echo "    build        Build both the crate and accompanying book"
	-@ echo "    word-count   Get some (rough) statistics about the repository"
	-@ echo "    clean        Remove any unnecessary files and build artefacts"
	-@ echo "    usage        Print this help text"

open: build
	cargo doc --open
	$OPEN ./book/index.html

build: build-crate build-docs

build-crate:
	tango
	cargo build

build-docs:
	mdbook build

clean:
	cargo clean 
	mdbook clean
	git clean -f -x -d 

word-count:
	-@ wc --words $$(find src/ -name "*.md")
