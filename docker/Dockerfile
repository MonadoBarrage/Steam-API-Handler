FROM rust:latest

WORKDIR /usr/src/Steam-API-Handler
COPY ../ .

RUN useradd app
USER app
EXPOSE 9116

RUN cargo build

CMD ["cargo","run"]
