#!/bin/bash
#
# Build configuration for test_concat.
#
export PACKAGE_NAME=test_concat

export BUILD_COMMAND=(
  # env
  # GITHUB_ACTION=build
  # scripts/setup_build_env.sh
  # '&&'
  scripts/bazel_build_target.sh
  --output_dir
  binaries
  //containers/test_concat:oci_runtime_bundle.tar
)

export SUBJECT_PATHS=(
  binaries/test_concat/container.tar
)
