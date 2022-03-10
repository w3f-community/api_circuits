################################################################################
# cargo install --path . --root target/install/
# mkdir ./target/install/lib/
# find target/release/build/ -type f -name "*.so*" -exec cp  {} ./target/install/lib/ \;
# docker build -f Dockerfile_M1.dockerfile -t api_circuits:dev .
# docker run -it --name api_circuits --rm -p 3000:3000 --env RUST_LOG="warn,info,debug" api_circuits:dev /usr/local/bin/api_circuits --ipfs-server-multiaddr /ip4/172.17.0.1/tcp/5001

FROM rust:1.59 as builder

ENV APP_NAME api_circuits

WORKDIR /usr/src/app

# TODO use this
# prereq: install CMake, Ninja, etc
# COPY . .
# RUN cargo install --path .

# directly copy the result of "cargo install" in the host local folder
COPY target/install/bin/$APP_NAME /usr/local/cargo/bin/$APP_NAME
# MUST also get all the shared libs
ADD target/install/lib /usr/local/lib/$APP_NAME/
# really UGLY; we should use CMake install...
ADD deps/lib_circuits/data /data/

################################################################################

FROM ubuntu:20.04

EXPOSE 3000

ENV APP_NAME api_circuits
ENV LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/lib

# TODO remove patchelf once proper PROD container
RUN apt-get update && apt-get install -y libboost-filesystem-dev libpng-dev libreadline-dev libtcl patchelf && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/lib/$APP_NAME /usr/local/lib/
COPY --from=builder /usr/local/cargo/bin/$APP_NAME /usr/local/bin/$APP_NAME
# TODO can this be removed in PROD?
COPY --from=builder /data /home/pratn/Documents/interstellar/api_circuits/deps/lib_circuits/data/
# # that is really ugly; we MUST fix some lib SONAME/path
RUN mv /usr/local/lib/libglog.so.0.6.0 /usr/local/lib/libglog.so.1
RUN mv /usr/local/lib/libfmt.so.8.1.1 /usr/local/lib/libfmt.so.8
# use patchelf instead b/c the default soname is eg (check with readelf -d libyosys.so)
#  0x000000000000000e (SONAME)             Library soname: [/home/.../api_circuits/target/release/build/lib-circuits-wrapper-7138103e084a25fc/out/build/_deps/yosys_dl-src/libyosys.so]
RUN patchelf --set-soname libyosys.so /usr/local/lib/libyosys.so
RUN patchelf --replace-needed $(ldd /usr/local/lib/libverilog_compiler.so | grep yosys | awk '{print $1}') libyosys.so /usr/local/lib/libverilog_compiler.so

CMD ["sh", "-c", "$APP_NAME"]