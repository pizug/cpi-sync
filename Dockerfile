FROM rust:1.58.1-alpine AS build
RUN apk add musl-dev openssl-dev
WORKDIR /code
RUN cargo new blank

COPY Cargo.toml ./blank/Cargo.toml

WORKDIR /code/blank
RUN cargo build --release

COPY . .

RUN cargo install --path .

FROM alpine:3.15.0
COPY --from=build /code/blank/target/release/cpi-sync /cpi-sync
ENTRYPOINT ["/cpi-sync"]
