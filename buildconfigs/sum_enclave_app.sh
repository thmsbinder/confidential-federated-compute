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
  scripts/cargo_build_target.sh
  --output_dir
  binaries
  sum_enclave_app
)

export SUBJECT_PATHS=(
  binaries/sum_example/binary
)
