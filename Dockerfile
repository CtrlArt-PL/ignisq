# STAGE 1: Builder
FROM rust:1.93-alpine AS builder

RUN apk add --no-cache musl-dev gcc

WORKDIR /app
COPY . .

RUN cargo build --release

# STAGE 2: Runtime
FROM alpine:3.19

RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app
COPY --from=builder /app/target/release/ignisq .

EXPOSE 9191
CMD ["./ignisq"]