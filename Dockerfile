FROM ubuntu:latest
USER root

RUN apt-get update
RUN apt-get install curl libclang-dev pkg-config build-essential -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

COPY ./ ./

RUN cargo build --release
EXPOSE 33533

ENTRYPOINT ["./target/release/node"]
