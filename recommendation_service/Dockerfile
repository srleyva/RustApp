FROM rust:latest as build

RUN mv /usr/bin/openssl ~/tmp
RUN wget https://www.openssl.org/source/openssl-1.1.0g.tar.gz && \
    tar -zxf openssl-1.1.0g.tar.gz && cd openssl-1.1.0g && \
    ./config && \
    make && \
    make install && \
    ln -s /usr/local/bin/openssl /usr/bin/openssl && \
    ldconfig

RUN rustup component add rustfmt

RUN cd / && USER=root cargo new playground

WORKDIR /playground
ADD Cargo.toml /playground/Cargo.toml
RUN cargo build
RUN cargo build --release
RUN rm src/*.rs

WORKDIR /usr/src/myapp

COPY Cargo.toml .
COPY src ./src

RUN cargo build --release

RUN cargo install --path .
