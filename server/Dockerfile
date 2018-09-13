FROM rust:1.28.0

RUN rustup update nightly
RUN rustup default nightly

RUN cargo install lazy_static --vers =1.1.0
RUN cargo install clap --vers =2.32.0
RUN cargo install slog --vers =2.3.3
RUN cargo install slog-term --vers =2.4.0
RUN cargo install slog-async --vers =2.3.0
RUN cargo install ws --vers =0.7.8
RUN cargo install futures --vers =0.1.23
RUN cargo install tokio-core --vers =0.1.17
RUN cargo install serde --vers =1.0
RUN cargo install serde_derive --vers =1.0
RUN cargo install serde_yaml --vers =0.8.3
RUN cargo install serde_json --vers =1.0.27
RUN cargo install dirs --vers =1.0.3
RUN cargo install rand --vers =0.5.5

COPY . /server
WORKDIR /server

RUN cargo build

ENTRYPOINT ["target/debug/rd-server", "-c /server/config"]
