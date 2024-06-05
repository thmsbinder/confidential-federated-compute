#!/bin/bash
#
# Build configuration for square_enclave_app.
#
export PACKAGE_NAME=square_example

export BUILD_COMMAND=(
  # env
  # GITHUB_ACTION=build
  # scripts/setup_build_env.sh
  # '&&'
  cargo
  build
  --release
  --package
  square_enclave_app
)

export SUBJECT_PATHS=(
  target/x86_64-unknown-none/release/square_enclave_app
)
