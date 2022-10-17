FROM rust:latest as builder
COPY . .
RUN cargo prisma generate && cargo build --release --package oxy-login

FROM debian:bullseye-slim
RUN apt-get update && apt-get -y install openssl libssl-dev
COPY --from=builder /target/release/oxy-login /usr/local/bin/oxy-login

EXPOSE 8484
CMD ["oxy-login"]

# TODO create individual dockerfiles in /oxy-login and /oxy-world, set base path on railway