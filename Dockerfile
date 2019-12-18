FROM rust
MAINTAINER Oleksii Filonenko <brightone@protonmail.com>

RUN rustup toolchain install stable

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD ["rusteam"]
