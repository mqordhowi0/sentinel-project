use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::time::Duration;
use tokio::time::sleep;
// Hapus baris "use warp..." karena kita tidak butuh server di worker

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // === BAGIAN DUMMY SERVER KITA HAPUS TOTAL ===
    // (Worker tidak perlu membuka port, dia cuma perlu jalan di background)
    
    // === Koneksi Database ===
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL wajib diisi!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("🤖 Sentinel Worker: SIAP MEMBURU LINK BERBAHAYA! (Mode Silent)");

    loop {
        // Cari link yang statusnya 'pending'
        let row = sqlx::query("SELECT id, url FROM links WHERE status = 'pending' LIMIT 1")
            .fetch_optional(&pool)
            .await?;

        match row {
            Some(record) => {
                let id: i32 = record.get("id");
                let url: String = record.get("url");

                println!("🔎 Sedang memeriksa: {}", url);
                let status_hasil = cek_link(&url);
                
                // Update status
                sqlx::query("UPDATE links SET status = $1 WHERE id = $2")
                    .bind(status_hasil.clone())
                    .bind(id)
                    .execute(&pool)
                    .await?;

                println!("✅ Selesai! Status diubah jadi: {}", status_hasil);
            }
            None => {
                // Kalau tidak ada tugas, tidur 3 detik
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

fn cek_link(url: &str) -> String {
    if url.contains("jahat") || url.contains("virus") {
        return "dangerous".to_string();
    }
    "safe".to_string()
}
