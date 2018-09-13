FROM rust:1.28.0

RUN rustup update nightly
RUN rustup default nightly

WORKDIR /

RUN USER=root cargo new server

COPY Cargo.toml_deps /server/Cargo.toml

WORKDIR /server

# Dummy build to get the dependencies cached.
RUN cargo build

RUN rm -f /server/src/main.rs

COPY . /server

WORKDIR /server

RUN cargo build

ENTRYPOINT ["target/debug/rd-server", "-c /server/config"]
