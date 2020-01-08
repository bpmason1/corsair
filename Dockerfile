FROM rust:1.40.0-alpine as builder

RUN if [ ! -d /tmp ]; then mkdir /tmp; fi

WORKDIR /tmp

COPY Cargo.toml .
COPY Cargo.lock .
COPY src ./src

RUN cargo build --release

#-------------------------------------------------------------
FROM alpine:3.11.2

RUN apk add --no-cache bash

COPY --from=builder /tmp/target/release/corsair .

COPY scripts ./scripts

# ENV LISTEN_ADDR=127.0.0.1:8000 PROXY_ADDR=127.0.0.1:4000
CMD ["./scripts/run"]

