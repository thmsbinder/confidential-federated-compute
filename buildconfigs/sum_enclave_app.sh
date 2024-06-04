#!/bin/bash
#
# Build configuration for sum_enclave_app.
#
export PACKAGE_NAME=sum_example

export BUILD_COMMAND=(
  # env
  # GITHUB_ACTION=build
  # scripts/setup_build_env.sh
  # '&&'
  cargo
  build
  --release
  --package
  sum_enclave_app
)

export SUBJECT_PATHS=(
  target/x86_64-unknown-none/release/sum_enclave_app
)
