use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use url::{Url, Host};

// Kita butuh 2 database di memori
struct SecurityDb {
    bad_domains: HashSet<String>, // Buat blokir website jahat full (misal: jahat.com)
    bad_urls: HashSet<String>,    // Buat blokir link spesifik di web baik (misal: github.com/virus)
}

type SharedDb = Arc<Mutex<SecurityDb>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL wajib diisi!");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("🌍 Mengunduh Data Intelijen Global...");
    let security_db = load_intelligence().await;
    
    {
        let db = security_db.lock().unwrap();
        println!("🔥 SIAP! Memuat {} Domain & {} Link Spesifik Berbahaya.", 
            db.bad_domains.len(), db.bad_urls.len());
    }

    loop {
        let row = sqlx::query("SELECT id, url FROM links WHERE status = 'pending' LIMIT 1")
            .fetch_optional(&pool)
            .await?;

        match row {
            Some(record) => {
                let id: i32 = record.get("id");
                let url_str: String = record.get("url");

                println!("🔎 Memeriksa: {}", url_str);
                
                // 1. Cek Database (Domain & URL Full)
                let mut status_hasil = cek_database(&url_str, &security_db);

                // 2. Cek Heuristik (IP Address)
                if status_hasil == "safe" {
                    status_hasil = cek_heuristik(&url_str);
                }

                sqlx::query("UPDATE links SET status = $1 WHERE id = $2")
                    .bind(&status_hasil)
                    .bind(id)
                    .execute(&pool)
                    .await?;

                println!("✅ Hasil: {} -> {}", url_str, status_hasil);
            }
            None => {
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

async fn load_intelligence() -> SharedDb {
    let mut domains = HashSet::new();
    let mut urls = HashSet::new();
    
    // 1. Download HOSTS (Daftar Domain)
    println!("   ↳ Downloading Domain Blacklist...");
    if let Ok(resp) = reqwest::get("https://urlhaus.abuse.ch/downloads/hostfile/").await {
        if let Ok(text) = resp.text().await {
            for line in text.lines() {
                if line.starts_with('#') || line.trim().is_empty() { continue; }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    domains.insert(parts[1].to_string());
                }
            }
        }
    }

    // 2. Download URLS (Daftar Link Spesifik - PENTING BUAT GITHUB!)
    println!("   ↳ Downloading Specific URL Blacklist...");
    if let Ok(resp) = reqwest::get("https://urlhaus.abuse.ch/downloads/text/").await {
        if let Ok(text) = resp.text().await {
            for line in text.lines() {
                if line.starts_with('#') || line.trim().is_empty() { continue; }
                // Di file ini, satu baris = satu link full
                urls.insert(line.trim().to_string());
            }
        }
    }

    Arc::new(Mutex::new(SecurityDb {
        bad_domains: domains,
        bad_urls: urls,
    }))
}

fn cek_database(input_url: &str, db: &SharedDb) -> String {
    let database = db.lock().unwrap();

    // Cek 1: Apakah link ini PERSIS SAMA dengan link jahat yang dikenal?
    // Ini yang akan menangkap link GitHub tadi!
    if database.bad_urls.contains(input_url) {
        println!("⚠️ BLOCKED: Link spesifik terdeteksi di database!");
        return "dangerous".to_string();
    }

    // Cek 2: Apakah domainnya jahat?
    if let Ok(parsed) = Url::parse(input_url) {
        if let Some(domain) = parsed.host_str() {
             if database.bad_domains.contains(domain) {
                println!("⚠️ BLOCKED: Domain {} ada di blacklist!", domain);
                return "dangerous".to_string();
            }
        }
    }

    "safe".to_string()
}

fn cek_heuristik(input_url: &str) -> String {
    if let Ok(parsed) = Url::parse(input_url) {
        if let Some(host) = parsed.host() {
            match host {
                Host::Ipv4(_) | Host::Ipv6(_) => {
                    println!("⚠️ HEURISTIC: Menggunakan Raw IP Address!");
                    return "dangerous".to_string();
                },
                _ => {}
            }
        }
    }
    "safe".to_string()
}