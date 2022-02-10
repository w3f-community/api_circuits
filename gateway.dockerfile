# Proxy HTTP<->gRPC using Envoy.
# https://github.com/grpc/grpc-web/blob/master/net/grpc/gateway/docker/envoy/Dockerfile
# NOT necessarily needed b/c tonic includes this via the crate tonic-web
# But can be useful for comparison, or other testing purposes.
#
# docker build -t envoy_gateway -f gateway.dockerfile .
# docker run -p 8080:8080 -p 9901:9901 -p 10000:10000 --name envoy_gateway --rm envoy_gateway
# [DEBUG] docker run -p 8080:8080 -p 9901:9901 -p 10000:10000 --name envoy_gateway --rm envoy_gateway -l off --component-log-level upstream:debug,connection:trace -c /etc/envoy/envoy.yaml

FROM envoyproxy/envoy:v1.20.0

COPY envoy.yaml /etc/envoy/envoy.yaml

CMD /usr/local/bin/envoy -c /etc/envoy/envoy.yaml -l trace --log-path /tmp/envoy_info.log