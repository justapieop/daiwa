FROM rust:1.92.0-alpine AS build-stage
WORKDIR /build
COPY . /build
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=build-stage /build/target/release/daiwa /app
CMD ["./daiwa"]