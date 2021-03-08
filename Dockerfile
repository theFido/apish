FROM rust:1.40 as builder
RUN mkdir /source
WORKDIR /source
COPY . .
RUN cargo build --release

FROM debian:buster-slim
RUN mkdir /app
WORKDIR /app
COPY --from=builder /source/target/release/apish .
ENTRYPOINT ["/app/apish"]
