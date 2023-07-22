# Tff Worker

This folder contains initial prototyping for building and running Docker images
that use TFF for use in Trusted Brella.

## Instructions

Installing Docker is a prerequisite for the commands in this README. Rootless
Docker is recommended for compatibility with Oak. See
[Oak's Rootless Docker](https://github.com/project-oak/oak/blob/main/docs/development.md#rootless-docker)
installation instructions.

Run from within this directory, the following command builds a Docker image
containing TFF:

```
docker build . -t tff
```

You can then execute the hello_world.py script in the Docker container:

```
docker run tff:latest python ./app/hello_world.py
```

If you are running low on disk space you should prune dangling images and
containers:

```
docker image prune
```

```
docker container prune
```

# Regenerate requirements.txt for the Hello World tff_worker

To ensure safe downloads in the case that an attacker compromises the PyPI
account of a library we depend on, we require hashes for all packages installed
by Pip. We use requirements.txt to specify dependencies needed by the docker
image along with their hashes.

To regenerate requirements.txt, run the gen_requirements.sh shell script. Note
that it is imperative that the resulting requirements.txt is checked in; if
generating the requirements were part of the docker build process of the
tff_worker, we wouldn't get the security benefits of using hashes.

## TFF Worker Pipeline Transform Server

The TFF Worker Pipeline Transform server will be a gRPC server running on Oak
Containers and implementing the pipeline_transform API so that the untrusted
application can instruct it to perform transformations on data using TFF.

For now, we can run a Python gRPC server as a docker container with a port
exposed to the host. For testing purposes, we run a C++ client on the host.
Using a C++ client makes it easier to build within a docker container but
produce a binary that can be run on the host, so that people who want to develop
using this repo don't need to have a particular version of bazel installed on
their host.

The following commands should all be executed from the root of the
confidential-federated-compute repository.

We use bazel from within a Docker container to build the C++ client via the
following steps:

Since we are using rootless docker, we need to make sure the user running docker
can write to any directories that are shared between the docker container and
the host.

```
chmod a+rw /tmp/build_output
```

```
chmod a+rw ../confidential-federated-compute
```

Next, start a shell from a docker container that contains dependencies necessary
to run Bazel.

```
docker run -it -e BAZEL_CXXOPTS="-std=c++14" -v "$(pwd)":"$(pwd)" -v /tmp/build_output:/tmp/build_output -w "$(pwd)" --entrypoint=/bin/bash gcr.io/bazel-public/bazel:latest
```

Within this shell, build the C++ binary for the Pipeline Transform client.

```
ubuntu@2fb777e52785:/src/workspace$ bazel --output_base=/tmp/build_output build tff_worker/client:pipeline_transform_cpp_client
```

At this point you can use ^D to exit the docker container shell. The C++ client
should now be runnable from the host.

```
bazel-bin/tff_worker/client/pipeline_transform_cpp_client
```

Since there is no server running, the C++ client is expected to produce output
like the following:

```
Starting RPC Client for Pipeline Transform Server.
RPC failed: 14: failed to connect to all addresses; last error: UNKNOWN: ipv4:127.0.0.1:50051: Failed to connect to remote host: Connection refused
```

Now, we build the Docker image that will run the Python server. The '.' argument
specifies the context that is available to access files that are used when
building the Docker image. Since the build process uses Bazel, the context needs
to include the workspace root, so it is important that this command is run from
the root of the confidential-federated-compute repo. Building the image may take
a while the first time it runs, but on subsequent runs parts of the image
building process will be cached.

```
docker build -f tff_worker/server/Dockerfile -t pipeline_transform_server .
```

Once the image has successfully built, you can run the server in the docker
container, publishing and mapping the gRPC server port so it can be accessed
from localhost:

```
docker run -i -p 127.0.0.1:50051:50051 pipeline_transform_server:latest &
```

Now the server should be running as a background job, so you can try running the
C++ client again:

```
bazel-bin/tff_worker/client/pipeline_transform_cpp_client
```

This time, it should produce the following output:

```
Starting RPC Client for Pipeline Transform Server.
RPC failed: 12: Method not implemented!
```

To bring the docker process back to the foreground in order to quit the server,
use the `fg` command.