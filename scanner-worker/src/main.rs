use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time::sleep;

// GANTI INI DENGAN CONNECTION STRING DARI NEON KAMU
const DATABASE_URL: &str = "postgres://neondb_owner:npg_J2KmDV7qTpay@ep-polished-heart-a1ohucn8-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require";

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1. Konek ke Database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DATABASE_URL)
        .await?;

    println!("🤖 Sentinel Worker: SIAP MEMBURU LINK BERBAHAYA!");

    // 2. Loop Abadi (Worker tidak pernah tidur)
    loop {
        // Cari link yang statusnya 'pending'
        let row = sqlx::query!(
            r#"SELECT id, url FROM links WHERE status = 'pending' LIMIT 1"#
        )
        .fetch_optional(&pool)
        .await?;

        match row {
            Some(record) => {
                println!("🔎 Sedang memeriksa: {}", record.url);
                
                // Simulasi scanning (cek apakah link aktif)
                let status_hasil = cek_link(&record.url);
                
                // Update status di database
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
                // Kalau tidak ada tugas, istirahat 3 detik
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

// Fungsi sederhana untuk cek link (Dummy Logic)
fn cek_link(url: &str) -> String {
    // Di dunia nyata, di sini kita pakai AI atau Blacklist check.
    // Untuk sekarang: Kalau ada kata "jahat", kita anggap dangerous.
    if url.contains("jahat") || url.contains("virus") {
        return "dangerous".to_string();
    }
    "safe".to_string()
}