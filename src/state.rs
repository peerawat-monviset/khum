use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::sync::{Mutex, RwLock};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct PromoCode {
    pub code: String,
    pub service: String,
    pub discount: f64,
}

pub struct CpuTracker {
    pub last_cpu_usec: u64,
    pub last_instant: Instant,
}

pub struct AppState {
    pub cache: RwLock<HashMap<String, PromoCode>>,
    pub db_path: &'static str,
    pub static_files: HashMap<&'static str, Vec<u8>>,
    pub cpu_tracker: Mutex<CpuTracker>,
    pub icon_urls: RwLock<HashMap<String, String>>,
}

impl AppState {
    pub fn new(db_path: &'static str) -> Self {
        let mut static_files = HashMap::new();
        static_files.insert("/index.html", fs::read("public/index.html").unwrap_or_default());
        static_files.insert("/main.css", fs::read("public/main.css").unwrap_or_default());
        static_files.insert("/main.js", fs::read("public/main.js").unwrap_or_default());

        let cpu_tracker = Mutex::new(CpuTracker {
            last_cpu_usec: crate::metrics::read_cpu_usec().unwrap_or(0),
            last_instant: Instant::now(),
        });

        AppState {
            cache: RwLock::new(HashMap::new()),
            db_path,
            static_files,
            cpu_tracker,
            icon_urls: RwLock::new(HashMap::new()),
        }
    }

    pub fn load_from_db(&self) -> io::Result<()> {
        if !std::path::Path::new(self.db_path).exists() {
            return Ok(());
        }
        let file = fs::File::open(self.db_path)?;
        let reader = BufReader::new(file);
        let mut cache = self.cache.write().unwrap();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() == 3 {
                let code = parts[0].to_string();
                let service = parts[1].to_string();
                if let Ok(discount) = parts[2].parse::<f64>() {
                    cache.insert(code.clone(), PromoCode { code, service, discount });
                }
            }
        }
        Ok(())
    }

    pub fn add_promo(&self, promo: PromoCode) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.db_path)?;
        
        writeln!(file, "{};{};{}", promo.code, promo.service, promo.discount)?;

        let mut cache = self.cache.write().unwrap();
        cache.insert(promo.code.clone(), promo);
        Ok(())
    }

    pub fn get_all_promos(&self) -> Vec<PromoCode> {
        let cache = self.cache.read().unwrap();
        cache.values().cloned().collect()
    }
}
