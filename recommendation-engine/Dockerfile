FROM rust:latest as build

RUN rustup component add rustfmt --toolchain 1.45.1-x86_64-unknown-linux-gnu

RUN cd / && USER=root cargo new playground

WORKDIR /playground
ADD Cargo.toml /playground/Cargo.toml
RUN touch ./src/client.rs && echo "fn main() { }" > ./src/client.rs
RUN touch ./src/server.rs && echo "fn main() { }" > ./src/server.rs
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