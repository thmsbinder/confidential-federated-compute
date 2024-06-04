#!/bin/bash
#
# Build configuration for sql_server.
#
export PACKAGE_NAME=sql_server

export BUILD_COMMAND=(
  # env
  # GITHUB_ACTION=build
  # scripts/setup_build_env.sh
  # '&&'
  scripts/bazel_build_target.sh
  --output_dir
  binaries
  //containers/sql_server:oci_runtime_bundle.tar
)

export SUBJECT_PATHS=(
  binaries/sql_server/container.tar
)
