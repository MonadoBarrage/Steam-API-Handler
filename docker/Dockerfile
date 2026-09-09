FROM rust:latest

WORKDIR /usr/src/Steam-API-Handler
COPY . .

RUN useradd -m app
USER app

RUN cargo build
EXPOSE 9116

CMD ["cargo", "run"]
