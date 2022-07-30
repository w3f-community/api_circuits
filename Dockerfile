################################################################################

# podman build -f Dockerfile -t api_circuits:dev -t ghcr.io/interstellar-network/api_circuits:dev --volume ~/.cargo:/root/.cargo:rw --volume $(pwd)/target/release:/usr/src/app/target/release:rw .
# NOTE: it CAN work with Docker but it less than ideal b/c it can not reuse the host's cache
# NOTE: when dev/test: if you get "ninja: error: loading 'build.ninja': No such file or directory"
# -> FIX: find target/release/ -type d -name "*-wrapper-*" -exec rm -rf {} \;
# b/c docker build has no support for volume contrary to podman/buildah
# docker run -it --name api_circuits --rm -p 3000:3000 --env RUST_LOG="warn,info,debug" api_circuits:dev /usr/local/bin/api_circuits --ipfs-server-multiaddr /ip4/172.17.0.1/tcp/5001

FROM ghcr.io/interstellar-network/ci-images/ci-base-rust:dev as builder

WORKDIR /usr/src/app

# "error: 'rustfmt' is not installed for the toolchain '1.59.0-x86_64-unknown-linux-gnu'"
RUN rustup component add rustfmt

# TODO install yosys.deb + abc.deb
# cf .github/workflows/rust.yml "Install dependencies" and "install "internal" dependencies"
RUN apt-get update && apt-get install -y \
    libboost-filesystem-dev && \
    wget https://github.com/Interstellar-Network/yosys/releases/download/yosys-0.15-interstellar/yosys-0.1.1-Linux.deb -O yosys.deb && \
    sudo apt-get install -y --no-install-recommends ./yosys.deb && \
    wget https://github.com/Interstellar-Network/abc/releases/download/0.0.1/abc-0.1.1-Linux.deb -O abc.deb && \
    sudo apt-get install -y --no-install-recommends ./abc.deb && \
    rm -rf /var/lib/apt/lists/*  && \
    rm ./yosys.deb ./abc.deb

COPY . .
# MUST use "--locked" else Cargo.lock is ignored
RUN cargo install --locked --path .

# MUST also get all the shared libs; using the CMake generated list of libs
# cf https://github.com/Interstellar-Network/cmake/blob/main/export_libs.cmake
# Typically it SHOULD be only libyosys b/c we build all STATIC except this one
# ls -al /usr/local/lib/
# total 17156
# drwxr-xr-x  3 root root     4096 Mar 22 12:06 .
# drwxr-xr-x 12 root root     4096 Mar 22 12:06 ..
# -rwxr-xr-x  1 root root 17551384 Mar 22 12:06 liblibyosys.so
# drwxr-xr-x  3 root root     4096 Mar 18 06:30 python3.9
# NOTE: if it fails with cp: will not overwrite just-created '/usr/local/lib/liblibyosys.so' with '/usr/src/app/target/release/build/lib-circuits-wrapper-a097322ac7999802/out/build/_deps/yosys_fetch-build/liblibyosys.so'
# It probably means you are caching the container target/ by using a volume and there are multiple build dir
# CHECK: find target/release/build/ -type d -name "*lib-circuits-wrapper*"
# If yes: DELETE the dup
RUN cat $(find target/release/ -type f -name cmake_generated_libs) | tr " " "\n" |  grep "/usr/src/app/target/release/.*.so" > list_local_shared_libs && \
    xargs --arg-file=list_local_shared_libs cp --target-directory=/usr/local/lib/ && \
    rm list_local_shared_libs \
    || echo "no shared libs to copy" && touch /usr/local/lib/no_shared_lib_to_copy

################################################################################

FROM ubuntu:20.04

EXPOSE 3000

ENV APP_NAME api_circuits

ENV LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/lib

# TODO libpng etc: use static in CMake and remove from here
RUN apt-get update && apt-get install -y libpng-dev libreadline-dev libtcl && rm -rf /var/lib/apt/lists/*

# NOTE if "no shared libs to copy" above; we  MUST add a random file else COPY fails with:
# "copier: stat: ["/usr/local/lib/*.so"]: no such file or directory"
# cf https://stackoverflow.com/questions/31528384/conditional-copy-add-in-dockerfile
COPY --from=builder /usr/local/lib/no_shared_lib_to_copy /usr/local/lib/*.so /usr/local/lib/
# TODO use ldd and make that fully dynamic?
COPY --from=builder /usr/lib/libyosys.so /usr/lib/libabc.so /usr/lib/
COPY --from=builder /usr/local/cargo/bin/$APP_NAME /usr/local/bin/$APP_NAME
# TODO use CMake install and DO NOT hardcode a path
COPY --from=builder /usr/src/app/lib_circuits_wrapper/deps/lib_circuits/data /usr/src/app/lib_circuits_wrapper/deps/lib_circuits/data/
# that is really ugly; we MUST fix some lib SONAME/path
# TODO TOREMOVE?
# RUN mv /usr/local/lib/libglog.so.0.6.0 /usr/local/lib/libglog.so.1
# RUN mv /usr/local/lib/libfmt.so.8.1.1 /usr/local/lib/libfmt.so.8
# use patchelf instead b/c the default soname is eg (check with readelf -d libyosys.so)
#  0x000000000000000e (SONAME)             Library soname: [/home/.../api_circuits/target/release/build/lib-circuits-wrapper-7138103e084a25fc/out/build/_deps/yosys_dl-src/libyosys.so]
# RUN patchelf --set-soname libyosys.so /usr/local/lib/libyosys.so
# RUN patchelf --replace-needed $(ldd /usr/local/lib/libverilog_compiler.so | grep yosys | awk '{print $1}') libyosys.so /usr/local/lib/libverilog_compiler.so

CMD ["sh", "-c", "$APP_NAME"]