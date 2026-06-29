use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::process::Command;
use std::collections::HashMap;
use crate::state::{AppState, PromoCode};

pub fn spawn_background_scraper(state: Arc<AppState>) {
    thread::spawn(move || {
        println!("Background worker: Initializing scraper...");
        loop {
            println!("Background worker: Fetching/updating delivery promo codes...");
            
            let mock_promos = vec![
                PromoCode { code: "GRAB60".to_string(), service: "grab".to_string(), discount: 60.0 },
                PromoCode { code: "LINEMANDEL".to_string(), service: "lineman".to_string(), discount: 30.0 },
                PromoCode { code: "RBH40".to_string(), service: "robinhood".to_string(), discount: 40.0 },
                PromoCode { code: "SHOPEE50".to_string(), service: "shopee".to_string(), discount: 50.0 },
            ];

            let mut to_add = Vec::new();
            {
                let cache = state.cache.read().unwrap();
                for promo in mock_promos {
                    if !cache.contains_key(&promo.code) {
                        to_add.push(promo);
                    }
                }
            }

            for promo in to_add {
                let code = promo.code.clone();
                if let Err(e) = state.add_promo(promo) {
                    eprintln!("Background worker Error: Failed to save promo: {}", e);
                } else {
                    println!("Background worker: Cached and persisted new code [{}]", code);
                }
            }

            update_app_store_icons(&state);

            thread::sleep(Duration::from_secs(600)); // Sleep 10 minutes
        }
    });
}

fn update_app_store_icons(state: &AppState) {
    println!("Background worker: Querying TH App Store for app icons...");
    let output = match Command::new("curl")
        .args(&[
            "-s",
            "https://itunes.apple.com/lookup?id=647268330,1076238296,1526791835,959841453&country=th"
        ])
        .output() {
            Ok(out) => out,
            Err(e) => {
                eprintln!("Background worker: Failed to execute curl for App Store search: {}", e);
                return;
            }
        };

    if !output.status.success() {
        eprintln!("Background worker: App Store curl lookup failed");
        return;
    }

    let json_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut map = HashMap::new();

    // Iterate through blocks starting with wrapperType (each result is a separate block)
    for block in json_str.split("\"wrapperType\"") {
        let track_id = if let Some(idx) = block.find("\"trackId\":") {
            let rest = &block[idx + 10..];
            let end_idx = rest.find(',').unwrap_or(rest.len());
            rest[..end_idx].trim().parse::<u64>().ok()
        } else {
            None
        };

        let artwork_url = if let Some(idx) = block.find("\"artworkUrl100\":\"") {
            let rest = &block[idx + 17..];
            let end_idx = rest.find('"').unwrap_or(rest.len());
            Some(rest[..end_idx].replace("\\/", "/"))
        } else if let Some(idx) = block.find("\"artworkUrl512\":\"") {
            let rest = &block[idx + 17..];
            let end_idx = rest.find('"').unwrap_or(rest.len());
            Some(rest[..end_idx].replace("\\/", "/"))
        } else {
            None
        };

        if let (Some(tid), Some(url)) = (track_id, artwork_url) {
            let key = match tid {
                647268330 => "grab",
                1076238296 => "lineman",
                1526791835 => "robinhood",
                959841453 => "shopee",
                _ => continue,
            };
            map.insert(key.to_string(), url);
        }
    }

    if !map.is_empty() {
        let mut icon_urls = state.icon_urls.write().unwrap();
        for (key, url) in map {
            println!("Background worker: Resolved App Store icon for [{}]: {}", key, url);
            icon_urls.insert(key, url);
        }
    }
}
