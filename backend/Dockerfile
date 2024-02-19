FROM rust:latest as build

WORKDIR /app
COPY ./Cargo.* .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build -r
RUN rm src/main.rs
COPY . .
RUN cargo build -r

FROM debian:bookworm
COPY --from=build /app/target/release/bingo_backend /usr/bin/bingo_backend
CMD [ "bingo_backend" ]
