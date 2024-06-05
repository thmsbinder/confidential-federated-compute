#!/usr/bin/env bash
#
# This script can be used by anyone, including CI, to pull a version of the
# image from Google Container Registry, which allows public downloads.
#
# See https://pantheon.corp.google.com/gcr/settings?project=oak-ci&folder&organizationId=433637338589
# See https://pantheon.corp.google.com/artifacts/docker/oak-ci/europe-west2/oak-development?project=oak-ci
readonly DOCKER_IMAGE_NAME=europe-west2-docker.pkg.dev/oak-ci/oak-development/oak-development:latest
readonly DOCKER_IMAGE_REPO_DIGEST=europe-west2-docker.pkg.dev/oak-ci/oak-development/oak-development@sha256:59f3914b8237601bcacdb6ba86c6aebe9f5fcc49c9ed377e281ed1e852bc7faa

docker pull "${DOCKER_IMAGE_NAME}"
docker pull "${DOCKER_IMAGE_REPO_DIGEST}"
