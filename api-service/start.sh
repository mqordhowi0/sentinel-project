#!/bin/sh

# 1. Jalankan Worker Rust di background (pake tanda &)
./scanner-worker &

# 2. Jalankan API Golang di foreground (biar container gak mati)
./server