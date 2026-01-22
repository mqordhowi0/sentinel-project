# === TAHAP 1: Masak Golang (API) ===
FROM golang:1.23-alpine AS go-builder
WORKDIR /app
COPY api-service/go.mod api-service/go.sum ./
RUN go mod download
COPY api-service/ .
RUN go build -o server main.go

# === TAHAP 2: Masak Rust (Worker) ===
# GANTI KE VERSI TERBARU (SLIM)
FROM rust:slim as rust-builder
WORKDIR /app
COPY scanner-worker/ .
# HAPUS KUNCI LAMA BIAR GA ERROR
RUN rm -f Cargo.lock
RUN cargo build --release

# === TAHAP 3: Sajikan (Final) ===
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=go-builder /app/server .
COPY --from=go-builder /app/start.sh .
COPY --from=rust-builder /app/target/release/scanner-worker .

RUN chmod +x start.sh
EXPOSE 8080
CMD ["./start.sh"]
