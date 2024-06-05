#!/bin/bash
#
# Build configuration for square_enclave_app.
#
export PACKAGE_NAME=square_example

export BUILD_COMMAND=(
  scripts/cargo_build_target.sh
  --output_dir
  binaries
  square_enclave_app
)

export SUBJECT_PATHS=(
  binaries/square_example/binary
)
