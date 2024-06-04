#!/bin/bash
#
# Build configuration for sum_enclave_app.
#
export PACKAGE_NAME=sum_example

export BUILD_COMMAND=(
  cargo
  build
  --release
  --package
  sum_enclave_app
)

export SUBJECT_PATHS=(
  target/x86_64-unknown-none/release/sum_enclave_app
)
