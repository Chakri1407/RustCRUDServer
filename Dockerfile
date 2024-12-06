# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# accept the build argument
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

COPY . .
RUN cargo build --release

# production stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin
COPY --from=builder /app/target/release/rust_crud_api .

CMD ["./rust_crud_api"]