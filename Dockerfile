#FROM rustlang/rust:nightly-slim as builder
FROM clux/muslrust:nightly as builder

COPY . .
RUN cargo build --release
RUN mkdir -p /build-out
RUN cp target/x86_64-unknown-linux-musl/release/eco-device-to-influxdb /build-out/
#RUN cp target/x86_64-unknown-linux-musl/debug/eco-device-to-influxdb /build-out/

FROM alpine

# Copy Application
COPY --from=builder /build-out/eco-device-to-influxdb /

# Add docker script to run
CMD /eco-device-to-influxdb