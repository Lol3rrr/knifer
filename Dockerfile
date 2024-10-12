FROM rust:1.81-bookworm

RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall trunk -y
RUN update-ca-certificates

RUN apt update
RUN apt install -y protobuf-compiler

WORKDIR /usr/src/knifer

COPY . /usr/src/knifer/

WORKDIR /usr/src/knifer/frontend

RUN mkdir ~/.ssh/
RUN ssh-keyscan -t rsa github.com >> ~/.ssh/known_hosts

RUN trunk build --release --locked -v

WORKDIR /usr/src/knifer/backend
RUN cargo build --release

EXPOSE 8000

WORKDIR /usr/src/knifer
CMD ["./target/release/backend"]
