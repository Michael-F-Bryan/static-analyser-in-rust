#!/bin/bash

set -ex


function install_dep() {
    local dep=$1

    if ! command -v $dep; then
        cargo install $dep
    fi
}


install_dep mdbook
install_dep tango
pip install --user ghp-import