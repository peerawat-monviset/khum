use std::sync::Arc;
use std::time::Duration;
use std::process::Command;
use std::collections::HashMap;
use crate::state::{AppState, PromoCode};

pub fn spawn_background_scraper(state: Arc<AppState>) {
    tokio::spawn(async move {
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

            tokio::time::sleep(Duration::from_secs(600)).await; // Sleep 10 minutes
        }
    });
}

fn update_app_store_icons(state: &AppState) {
    println!("Background worker: Querying TH/VN App Store for app icons...");
    let mut map = HashMap::new();

    // 1. Query TH Store for Grab, LINE MAN, and Robinhood
    if let Ok(output) = Command::new("curl")
        .args(&[
            "-s",
            "https://itunes.apple.com/lookup?id=647268330,1076238296,1526791835&country=th"
        ])
        .output() {
        if output.status.success() {
            if let Ok(json_str) = String::from_utf8(output.stdout) {
                parse_lookup_json(&json_str, &mut map);
            }
        }
    }

    // 2. Query VN Store for ShopeeFood (Customer App ID: 1137866760)
    if let Ok(output) = Command::new("curl")
        .args(&[
            "-s",
            "https://itunes.apple.com/lookup?id=1137866760&country=vn"
        ])
        .output() {
        if output.status.success() {
            if let Ok(json_str) = String::from_utf8(output.stdout) {
                parse_lookup_json(&json_str, &mut map);
            }
        }
    }

    // Write back to AppState
    let mut icon_urls = state.icon_urls.write().unwrap();
    for (k, v) in map {
        if icon_urls.get(&k) != Some(&v) {
            println!("Background worker: Resolved App Store icon for [{}]: {}", k, v);
            icon_urls.insert(k, v);
        }
    }
}

fn parse_lookup_json(json: &str, map: &mut HashMap<String, String>) {
    let mut current_pos = 0;
    while let Some(idx) = json[current_pos..].find("\"artworkUrl512\"") {
        let absolute_idx = current_pos + idx;
        let search_slice = &json[absolute_idx..];
        if let Some(val_start) = search_slice.find(':') {
            let val_slice = &search_slice[val_start + 1..];
            if let Some(quote_start) = val_slice.find('"') {
                if let Some(quote_end) = val_slice[quote_start + 1..].find('"') {
                    let mut url = val_slice[quote_start + 1..quote_start + 1 + quote_end].to_string();
                    url = url.replace("\\/", "/");
                    
                    // Replace /512x512bb.jpg suffix with /1024x1024bb.jpg for maximum quality
                    if url.contains("512x512bb.jpg") {
                        url = url.replace("512x512bb.jpg", "1024x1024bb.jpg");
                    } else if url.contains("100x100bb.jpg") {
                        url = url.replace("100x100bb.jpg", "1024x1024bb.jpg");
                    }

                    // Map specific app bundles to delivery providers
                    if url.contains("grab") || url.contains("Grab") {
                        map.insert("grab".to_string(), url);
                    } else if url.contains("lineman") || url.contains("Lineman") || url.contains("bb64a022-4c96-3849-63b5-63062204057e") {
                        map.insert("lineman".to_string(), url);
                    } else if url.contains("robinhood") || url.contains("Robinhood") || url.contains("41064f9c-e4ed-179a-1a1f-8beddca689a8") {
                        map.insert("robinhood".to_string(), url);
                    } else if url.contains("shopee") || url.contains("Shopee") || url.contains("d114731d-f283-4273-762d-97d35644434c") {
                        map.insert("shopee".to_string(), url);
                    }
                }
            }
        }
        current_pos = absolute_idx + 15;
    }
}
