package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"net/http"

	"github.com/lib/pq" // Driver Postgres
	"github.com/rs/cors" // Library CORS (Satpam Pintu)
)

// Connection string database Neon kamu
const connStr = "postgresql://neondb_owner:npg_J2KmDV7qTpay@ep-polished-heart-a1ohucn8.ap-southeast-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require"

type ScanRequest struct {
	URL string `json:"url"`
}

var db *sql.DB

func main() {
	var err error
	// Buka koneksi ke Database
	db, err = sql.Open("postgres", connStr)
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	// Cek koneksi
	if err = db.Ping(); err != nil {
		log.Fatal("Gagal konek DB: ", err)
	}

	// === ROUTER BARU (MUX) ===
	// Kita bikin router sendiri biar bisa dibungkus CORS
	mux := http.NewServeMux()
	
	mux.HandleFunc("/", homeHandler)
	mux.HandleFunc("/scan", scanHandler)   // POST: Input Link
	mux.HandleFunc("/check", checkHandler) // GET: Cek Status

	// === SETTING CORS (PENTING BUAT FRONTEND) ===
	c := cors.New(cors.Options{
		AllowedOrigins: []string{"*"}, // Boleh diakses dari mana aja (termasuk localhost)
		AllowedMethods: []string{"GET", "POST", "OPTIONS"},
		AllowedHeaders: []string{"Content-Type", "Authorization"},
		Debug:          true, // Nyalakan log CORS biar ketahuan kalau ada error
	})

	// Bungkus router kita dengan CORS handler
	handler := c.Handler(mux)

	fmt.Println("🚀 Server Sentinel siap di port 8080...")
	// Jalankan server dengan handler yang sudah ada CORS-nya
	log.Fatal(http.ListenAndServe(":8080", handler))
}

func homeHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"message": "Sentinel API Online 🛡️"})
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

	// Masukkan ke DB dengan status 'pending'
	sqlStatement := `INSERT INTO links (url) VALUES ($1) RETURNING id`
	var id int
	err := db.QueryRow(sqlStatement, req.URL).Scan(&id)
	
	// Handle error spesifik Postgres (misal duplicate key) bisa disini
	if err != nil {
		if pqErr, ok := err.(*pq.Error); ok {
			log.Println("DB Error:", pqErr.Message)
		}
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

func checkHandler(w http.ResponseWriter, r *http.Request) {
	id := r.URL.Query().Get("id")
	if id == "" {
		http.Error(w, "Parameter ID harus ada", http.StatusBadRequest)
		return
	}

	var url, status string
	err := db.QueryRow("SELECT url, status FROM links WHERE id=$1", id).Scan(&url, &status)
	if err == sql.ErrNoRows {
		http.Error(w, "ID tidak ditemukan", http.StatusNotFound)
		return
	} else if err != nil {
		http.Error(w, "Error database: "+err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"id":     id,
		"url":    url,
		"status": status,
	})
}