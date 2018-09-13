FROM rust:1.28.0

COPY . /server
WORKDIR /server

RUN rustup update nightly
RUN rustup default nightly

RUN cargo build

ENTRYPOINT ["target/debug/rd-server", "-c /server/config"]
