OPEN := xdg-open


usage:
	-@ echo "Usage:"
	-@ echo "    open         Build the project and open the tutorial in your browser"
	-@ echo "    build        Build both the crate and accompanying book"
	-@ echo "    word-count   Get some (rough) statistics about the repository"
	-@ echo "    clean        Remove any unnecessary files and build artefacts"
	-@ echo "    todo         Find all sections marked TODO or FIXME"
	-@ echo "    usage        Print this help text"

open: build
	cargo doc --open
	${OPEN} target/book/index.html

build: build-crate build-docs

build-crate: src/lib.rs
	cargo build

build-docs:
	bash -c 'type mdbook' || ${MAKE} try-install target=mdbook
	mdbook build

clean:
	cargo clean
	git clean -f -x -d

word-count:
	-@ echo -e "lines words file"
	-@ echo -e "----- ----- ----"
	-@ wc --lines --words $$(find src/ -name "*.md")

todo:
	bash -c 'type rg' || ${MAKE} try-install target=ripgrep
	rg 'TODO|FIXME' --iglob '*.md'

src/lib.rs:
	bash -c 'type tango' || ${MAKE} try-install target=tango
	tango

try-install:
	test -n "${target}" || ( echo must supply target; exit 1 )
	export ans=`bash -c 'read -p "Install ${target} with cargo (N/y)? " ans; \
		ans=$${ans:-n}; echo $${ans:0:1} | tr "[:upper:]" "[:lower:]"'` && \
		test "$$ans" = "y" && \
			cargo install ${target} || \
			exit 10
