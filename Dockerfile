# Build stage
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev protobuf-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src
COPY proto ./proto
COPY build.rs ./

# Build release binary
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/centy-daemon /app/centy-daemon

# Create data directory
RUN mkdir -p /data

ENV CENTY_DAEMON_ADDR=0.0.0.0:50051

EXPOSE 50051

CMD ["/app/centy-daemon"]
