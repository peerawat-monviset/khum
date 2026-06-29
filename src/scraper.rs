use std::sync::Arc;
use std::thread;
use std::time::Duration;
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

            thread::sleep(Duration::from_secs(600)); // Sleep 10 minutes
        }
    });
}
