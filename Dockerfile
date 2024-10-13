FROM rust:1.81-bookworm AS builder

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

WORKDIR /usr/src/knifer/

ENV FRONTEND_DIST_DIR=/root/knifer/static
RUN cargo build --release -p backend

FROM debian:bookworm

#
RUN apt update
Run apt install libssl3 ca-certificates -y

WORKDIR /root/knifer

COPY --from=builder /usr/src/knifer/frontend/dist/ /root/knifer/static
COPY --from=builder /usr/src/knifer/target/release/backend /root/knifer/backend

EXPOSE 3000

WORKDIR /root/knifer
ENTRYPOINT ["/root/knifer/backend"]
