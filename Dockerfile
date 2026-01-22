# === TAHAP 1: Masak Golang (API) ===
FROM golang:1.23-alpine AS go-builder
WORKDIR /app
# Ambil bahan dari folder api-service
COPY api-service/go.mod api-service/go.sum ./
RUN go mod download
COPY api-service/ .
RUN go build -o server main.go

# === TAHAP 2: Masak Rust (Worker) ===
FROM rust:1.75-slim-bookworm as rust-builder
WORKDIR /app
# Ambil bahan dari folder scanner-worker
COPY scanner-worker/ .
RUN cargo build --release

# === TAHAP 3: Sajikan (Final) ===
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Ambil hasil masakan tadi
COPY --from=go-builder /app/server .
COPY --from=go-builder /app/start.sh .
COPY --from=rust-builder /app/target/release/scanner-worker .

# Beri izin script mandor
RUN chmod +x start.sh

# Buka pintu 8080
EXPOSE 8080

# Jalankan keduanya!
CMD ["./start.sh"]
