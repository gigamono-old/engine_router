FROM rust:1.31

WORKDIR /usr/src/engine-api
COPY . .

RUN cargo install --path .

CMD ["server"]
