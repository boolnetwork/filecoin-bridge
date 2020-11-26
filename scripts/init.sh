#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
   rustup update nightly
   rustup update stable
fi

# Installation dependency of filecoin
sudo apt install hwloc libhwloc-dev

rustup target add wasm32-unknown-unknown --toolchain nightly
