package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"net/http"

	_ "github.com/lib/pq"
)

// GANTI connection string ini dengan milikmu (yang DIRECT/tanpa pooler lebih aman)
const connStr = "postgresql://neondb_owner:npg_J2KmDV7qTpay@ep-polished-heart-a1ohucn8.ap-southeast-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require"

type ScanRequest struct {
	URL string `json:"url"`
}

var db *sql.DB

func main() {
	var err error
	db, err = sql.Open("postgres", connStr)
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	if err = db.Ping(); err != nil {
		log.Fatal("Gagal konek DB: ", err)
	}

	// === DAFTAR RUTE ===
	http.HandleFunc("/", homeHandler)
	http.HandleFunc("/scan", scanHandler)   // POST: Kirim Link
	http.HandleFunc("/check", checkHandler) // GET: Cek Hasil Scan (BARU!)

	fmt.Println("🚀 Server Sentinel siap di port 8080...")
	http.ListenAndServe(":8080", nil)
}

func homeHandler(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "Sentinel API Online.")
}

func scanHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method harus POST", http.StatusMethodNotAllowed)
		return
	}

	var req ScanRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "JSON tidak valid", http.StatusBadRequest)
		return
	}

	sqlStatement := `INSERT INTO links (url) VALUES ($1) RETURNING id`
	var id int
	err := db.QueryRow(sqlStatement, req.URL).Scan(&id)
	if err != nil {
		http.Error(w, "Gagal simpan ke DB: "+err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"message": "Link diterima antrian",
		"id":      id,
		"status":  "pending",
	})
	fmt.Printf("📥 Link baru masuk: %s (ID: %d)\n", req.URL, id)
}

// === FITUR BARU: CEK STATUS ===
func checkHandler(w http.ResponseWriter, r *http.Request) {
	// Ambil ID dari URL (contoh: /check?id=1)
	id := r.URL.Query().Get("id")
	if id == "" {
		http.Error(w, "Parameter ID harus ada", http.StatusBadRequest)
		return
	}

	var url, status string
	// Cari di database
	err := db.QueryRow("SELECT url, status FROM links WHERE id=$1", id).Scan(&url, &status)
	if err == sql.ErrNoRows {
		http.Error(w, "ID tidak ditemukan", http.StatusNotFound)
		return
	} else if err != nil {
		http.Error(w, "Error database: "+err.Error(), http.StatusInternalServerError)
		return
	}

	// Tampilkan hasil
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"id":     id,
		"url":    url,
		"status": status, // Ini yang paling penting!
	})
}