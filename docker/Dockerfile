FROM rust:latest

RUN useradd app
USER app

WORKDIR /usr/src/Steam-API-Handler
COPY . .

RUN cargo build
EXPOSE 9116

CMD ["cargo", "run"]
