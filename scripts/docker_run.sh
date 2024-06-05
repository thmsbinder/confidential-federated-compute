#!/usr/bin/env bash
#
# Runs the provided command in the Docker container specified by
# DOCKER_IMAGE_ID.

readonly DOCKER_IMAGE_ID=sha256:0eaee35dde1820758b59c6c92069e1b23f236d74120e19168134380cd4e97f7b

docker_run_flags=(
  --rm
  --tty
  --env=CI
  --env=KOKORO_ARTIFACTS_DIR
  --env=TERM=xterm-256color
  --env=OAK_KVM_TESTS
  --env=RUST_BACKTRACE
  --env=RUST_LOGS
  "--volume=$PWD/bazel-cache:/home/docker/.cache/bazel"
  "--volume=$PWD/cargo-cache:/home/docker/.cargo"
  "--volume=$PWD:/workspace"
  # The container uses the host docker daemon, so docker commands running in
  # the container actually access the host filesystem. Thus mount the /tmp
  # directory as a volume in the container so that it can access the outputs of
  # docker commands that write to /tmp.
  --volume=/tmp:/tmp
  --volume=/dev/log:/dev/log
  --volume=/lib/modules:/lib/modules:ro
  --workdir=/workspace
  --network=host
  --security-opt=seccomp=unconfined
)

# Some CI systems (GitHub actions) do not run with an interactive TTY attached.
# When a CI environment variable is present, assume that we only have basic
# log output.
if [[ -n "${CI:-}" ]]; then
  docker_run_flags+=( --env=NO_COLOR=true )
else
  docker_run_flags+=( --interactive )
fi

# Run the provided command.
docker run "${docker_run_flags[@]}" "${DOCKER_IMAGE_ID}" "$@"
