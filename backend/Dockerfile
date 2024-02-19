FROM rust:latest as build

WORKDIR /app
COPY . .
RUN cargo fetch
RUN cargo build -r

FROM debian:bookworm
COPY --from=build /app/target/release/bingo_backend /usr/bin/bingo_backend
CMD [ "bingo_backend" ]
