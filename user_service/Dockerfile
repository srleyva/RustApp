FROM rust:latest as build

RUN rustup component add rustfmt

RUN cd / && USER=root cargo new playground

WORKDIR /playground
ADD Cargo.toml /playground/Cargo.toml
RUN mkdir -p ./src/bin
RUN touch ./src/bin/client.rs && echo "fn main() { }" > ./src/bin/client.rs
RUN touch ./src/bin/server.rs && echo "fn main() { }" > ./src/bin/server.rs
RUN cargo build
RUN cargo build --release
RUN rm src/*.rs

WORKDIR /usr/src/myapp

COPY Cargo.toml .
COPY build.rs ./
COPY src ./src
COPY ./proto ./proto

RUN cargo build --release

RUN cargo install --path .

FROM ubuntu

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y locales openssl ca-certificates

COPY --from=build /usr/local/cargo/bin/server /server
COPY --from=build /usr/local/cargo/bin/client /client

ENTRYPOINT ["/server"]