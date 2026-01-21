use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time::sleep;
use warp::Filter; // Library buat server palsu

// Pastikan DATABASE_URL nanti diambil dari Environment Variable
// (Kita tidak perlu hardcode di sini lagi karena nanti diset di Koyeb)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === BAGIAN 1: SIAPKAN SERVER PALSU (TOPENG) ===
    // Ini supaya Koyeb melihat aplikasi kita "Healthy" di port 8080
    let health_route = warp::any().map(|| "Worker is running safely!");
    
    // Jalankan server di background (tanpa menghentikan scanning)
    tokio::spawn(async move {
        println!("🎭 Dummy Server jalan di port 8080");
        warp::serve(health_route).run(([0, 0, 0, 0], 8080)).await;
    });

    // === BAGIAN 2: LOGIKA SCANNER ASLI ===
    // Ambil URL database dari Environment Variable (Settingan Koyeb)
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL wajib diisi!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("🤖 Sentinel Worker: SIAP MEMBURU LINK BERBAHAYA!");

    loop {
        let row = sqlx::query!(
            r#"SELECT id, url FROM links WHERE status = 'pending' LIMIT 1"#
        )
        .fetch_optional(&pool)
        .await?;

        match row {
            Some(record) => {
                println!("🔎 Sedang memeriksa: {}", record.url);
                let status_hasil = cek_link(&record.url);
                
                sqlx::query!(
                    r#"UPDATE links SET status = $1 WHERE id = $2"#,
                    status_hasil,
                    record.id
                )
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