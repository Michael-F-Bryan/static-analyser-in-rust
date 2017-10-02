#!/bin/bash

set -ex

# Only upload the built book to github pages if it's a commit to master
if [ "$TRAVIS_BRANCH" = master -a "$TRAVIS_PULL_REQUEST" = false ]; then
  cargo doc -v
  mdbook build 

  mkdir tmp
  mv target/doc tmp/doc
  mv target/book tmp/book
  echo '<html><meta http-equiv="refresh" content="0; URL=book/index.html"></html>' > tmp/index.html

  ghp-import -n tmp 
  git push -fq "https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git" gh-pages
fi