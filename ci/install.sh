#!/bin/bash

set -ex

dependencies=(mdbook tango)

function install_dep() {
    local dep=$1

    if ! command -v $dep; then
        cargo install $dep
    fi
}


for dependency in $dependencies; do
    install_dep $dependency
done


pip install --user ghp-import