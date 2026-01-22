# ========================
# TAHAP 1: Masak Golang (API)
# ========================
FROM golang:1.23-alpine AS go-builder
WORKDIR /app
COPY api-service/go.mod api-service/go.sum ./
RUN go mod download
COPY api-service/ .
RUN go build -o server main.go

# ========================
# TAHAP 2: Masak Rust (Worker)
# ========================
# Kita pakai 'slim' (versi terbaru) biar ga error Lockfile Version
FROM rust:slim as rust-builder
WORKDIR /app

# 🔥 INI KUNCI SUKSESNYA (Saran dari tetangga + temuan kita)
# Kita install alat bantu pkg-config dan OpenSSL
RUN apt-get update && apt-get install -y pkg-config libssl-dev

COPY scanner-worker/ .

# Kita hapus Lockfile biar dia generate ulang yang cocok sama server
# (Ini langkah aman karena kita ganti-ganti versi terus dari tadi)
RUN rm -f Cargo.lock

# Mulai Memasak!
RUN cargo build --release

# ========================
# TAHAP 3: Sajikan (Final Image)
# ========================
FROM debian:bookworm-slim
WORKDIR /app

# Install OpenSSL di piring saji juga (Runtime)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Ambil hasil masakan
COPY --from=go-builder /app/server .
COPY --from=go-builder /app/start.sh .
COPY --from=rust-builder /app/target/release/scanner-worker .

RUN chmod +x start.sh
EXPOSE 8080
CMD ["./start.sh"]
