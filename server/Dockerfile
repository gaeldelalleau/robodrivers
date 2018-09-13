FROM rust:1.28.0

RUN rustup update nightly
RUN rustup default nightly

RUN cargo build

COPY . /server
WORKDIR /server

ENTRYPOINT ["target/debug/rd-server", "-c /server/config"]
