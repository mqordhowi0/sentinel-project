use sqlx::postgres::PgPoolOptions;
use sqlx::Row; // PENTING: Kita butuh ini buat baca data manual
use std::time::Duration;
use tokio::time::sleep;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === Bagian 1: Server Palsu (Biar Koyeb Happy) ===
    let health_route = warp::any().map(|| "Worker is running safely!".to_string());
    
    tokio::spawn(async move {
        println!("🎭 Dummy Server jalan di port 8080");
        warp::serve(health_route).run(([0, 0, 0, 0], 8080)).await;
    });

    // === Bagian 2: Koneksi Database ===
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL wajib diisi!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("🤖 Sentinel Worker: SIAP MEMBURU LINK BERBAHAYA!");

    loop {
        // === PERUBAHAN PENTING DI SINI ===
        // Kita pakai sqlx::query (bukan query!) supaya tidak perlu konek DB saat build
        let row = sqlx::query("SELECT id, url FROM links WHERE status = 'pending' LIMIT 1")
            .fetch_optional(&pool)
            .await?;

        match row {
            Some(record) => {
                // Ambil data secara manual
                let id: i32 = record.get("id");
                let url: String = record.get("url");

                println!("🔎 Sedang memeriksa: {}", url);
                let status_hasil = cek_link(&url);
                
                // Update status juga pakai query biasa (tanpa tanda seru)
                sqlx::query("UPDATE links SET status = $1 WHERE id = $2")
                    .bind(status_hasil.clone())
                    .bind(id)
                    .execute(&pool)
                    .await?;

                println!("✅ Selesai! Status diubah jadi: {}", status_hasil);
            }
            None => {
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
