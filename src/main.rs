use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug)]
struct PromoCode {
    code: String,
    service: String,
    discount: f64,
}

struct AppState {
    cache: RwLock<HashMap<String, PromoCode>>,
    db_path: &'static str,
    static_files: HashMap<&'static str, Vec<u8>>,
}

impl AppState {
    fn new(db_path: &'static str) -> Self {
        let mut static_files = HashMap::new();
        // Cache static files in memory on startup to completely bypass disk I/O on client requests
        static_files.insert("/index.html", fs::read("public/index.html").unwrap_or_default());
        static_files.insert("/main.css", fs::read("public/main.css").unwrap_or_default());
        static_files.insert("/main.js", fs::read("public/main.js").unwrap_or_default());

        AppState {
            cache: RwLock::new(HashMap::new()),
            db_path,
            static_files,
        }
    }

    fn load_from_db(&self) -> io::Result<()> {
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

    fn add_promo(&self, promo: PromoCode) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.db_path)?;
        
        writeln!(file, "{};{};{}", promo.code, promo.service, promo.discount)?;

        let mut cache = self.cache.write().unwrap();
        cache.insert(promo.code.clone(), promo);
        Ok(())
    }

    fn get_all_promos(&self) -> Vec<PromoCode> {
        let cache = self.cache.read().unwrap();
        cache.values().cloned().collect()
    }
}

fn spawn_background_scraper(state: Arc<AppState>) {
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

fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let state = Arc::new(AppState::new("promos.txt"));
    if let Err(e) = state.load_from_db() {
        eprintln!("Failed to load database: {}", e);
    }

    spawn_background_scraper(Arc::clone(&state));

    let listener = TcpListener::bind(&addr).expect("Failed to bind port");
    println!("Server running on http://{}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state_clone = Arc::clone(&state);
                thread::spawn(move || {
                    handle_connection(stream, state_clone);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, state: Arc<AppState>) {
    let mut request_bytes = Vec::new();
    let mut buffer = [0; 512];
    
    // Dynamically read request headers from TCP socket until double CRLF is found
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                request_bytes.extend_from_slice(&buffer[..n]);
                if request_bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
                // Safety guard: Limit maximum request header size to 8KB
                if request_bytes.len() >= 8192 {
                    send_response(&mut stream, "HTTP/1.1 431 Request Header Fields Too Large\r\n\r\n", None);
                    return;
                }
            }
            Err(e) => {
                eprintln!("Failed to read stream: {}", e);
                return;
            }
        }
    }

    let request_str = String::from_utf8_lossy(&request_bytes);
    let mut lines = request_str.lines();
    let request_line = match lines.next() {
        Some(line) => line,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let full_path = parts[1];

    if method != "GET" {
        send_response(&mut stream, "HTTP/1.1 405 Method Not Allowed\r\n\r\n", None);
        return;
    }

    let (path, query) = if let Some(idx) = full_path.find('?') {
        (&full_path[..idx], &full_path[idx + 1..])
    } else {
        (full_path, "")
    };

    match path {
        "/" | "/index.html" => {
            serve_cached_file(&mut stream, &state.static_files["/index.html"], "text/html; charset=utf-8");
        }
        "/main.css" => {
            serve_cached_file(&mut stream, &state.static_files["/main.css"], "text/css; charset=utf-8");
        }
        "/main.js" => {
            serve_cached_file(&mut stream, &state.static_files["/main.js"], "application/javascript; charset=utf-8");
        }
        "/api/calculate" => {
            handle_calculate(&mut stream, query, state);
        }
        "/api/metrics" => {
            handle_metrics(&mut stream);
        }
        _ => {
            send_response(&mut stream, "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n", None);
        }
    }
}

fn serve_cached_file(stream: &mut TcpStream, contents: &[u8], content_type: &str) {
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        content_type,
        contents.len()
    );
    send_response(stream, &headers, Some(contents));
}

fn handle_calculate(stream: &mut TcpStream, query: &str, state: Arc<AppState>) {
    let mut basket: f64 = 250.0;
    let mut distance: f64 = 3.5;

    for param in query.split('&') {
        let kv: Vec<&str> = param.split('=').collect();
        if kv.len() == 2 {
            match kv[0] {
                "basket" => {
                    if let Ok(val) = kv[1].parse::<f64>() {
                        basket = val;
                    }
                }
                "distance" => {
                    if let Ok(val) = kv[1].parse::<f64>() {
                        distance = val;
                    }
                }
                _ => {}
            }
        }
    }

    let promos = state.get_all_promos();

    // Grab
    let grab_del = 10.0 + (distance * 8.0);
    let grab_disc = promos.iter()
        .filter(|p| p.service == "grab")
        .map(|p| {
            if p.code == "GRAB60" && basket >= 200.0 { 60.0 } else { p.discount }
        })
        .fold(0.0, f64::max);
    let grab_final = (basket + grab_del - grab_disc).max(0.0);

    // LINE MAN
    let lineman_del = distance * 12.0;
    let lineman_disc = promos.iter()
        .filter(|p| p.service == "lineman")
        .map(|p| {
            if p.code == "LINEMANDEL" { 30.0 } else { p.discount }
        })
        .fold(0.0, f64::max);
    let lineman_final = (basket + (lineman_del - lineman_disc).max(0.0)).max(0.0);

    // Robinhood
    let robinhood_del = 20.0 + (distance * 4.0);
    let robinhood_disc = promos.iter()
        .filter(|p| p.service == "robinhood")
        .map(|p| {
            if p.code == "RBH40" && basket >= 180.0 { 40.0 } else { p.discount }
        })
        .fold(0.0, f64::max);
    let robinhood_final = (basket + robinhood_del - robinhood_disc).max(0.0);

    // ShopeeFood
    let shopee_del = 10.0 + (distance * 6.0);
    let shopee_disc = promos.iter()
        .filter(|p| p.service == "shopee")
        .map(|p| {
            if p.code == "SHOPEE50" && basket >= 220.0 { 50.0 } else { p.discount }
        })
        .fold(0.0, f64::max);
    let shopee_final = (basket + shopee_del - shopee_disc).max(0.0);

    // Determine Best
    let mut best = "grab";
    let mut min_val = grab_final;

    if lineman_final < min_val {
        best = "lineman";
        min_val = lineman_final;
    }
    if robinhood_final < min_val {
        best = "robinhood";
        min_val = robinhood_final;
    }
    if shopee_final < min_val {
        best = "shopee";
    }

    let json = format!(
        "{{\"best\":\"{}\",\"providers\":[\
            {{\"key\":\"grab\",\"name\":\"GrabFood\",\"final\":{},\"original\":{}}},\
            {{\"key\":\"lineman\",\"name\":\"LINE MAN\",\"final\":{},\"original\":{}}},\
            {{\"key\":\"robinhood\",\"name\":\"Robinhood\",\"final\":{},\"original\":{}}},\
            {{\"key\":\"shopee\",\"name\":\"ShopeeFood\",\"final\":{},\"original\":{}}}\
        ]}}",
        best,
        grab_final, basket + grab_del,
        lineman_final, basket + lineman_del,
        robinhood_final, basket + robinhood_del,
        shopee_final, basket + shopee_del
    );

    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        json.len()
    );

    send_response(stream, &headers, Some(json.as_bytes()));
}

fn send_response(stream: &mut TcpStream, headers: &str, body: Option<&[u8]>) {
    if let Err(e) = stream.write_all(headers.as_bytes()) {
        eprintln!("Failed to write headers: {}", e);
        return;
    }
    if let Some(contents) = body {
        if let Err(e) = stream.write_all(contents) {
            eprintln!("Failed to write body: {}", e);
        }
    }
}

fn handle_metrics(stream: &mut TcpStream) {
    let mem_current = read_cgroup_metric("/sys/fs/cgroup/memory.current")
        .or_else(|_| read_cgroup_metric("/sys/fs/cgroup/memory/memory.usage_in_bytes"))
        .unwrap_or(0);

    let mem_limit = read_cgroup_metric("/sys/fs/cgroup/memory.max")
        .or_else(|_| read_cgroup_metric("/sys/fs/cgroup/memory/memory.limit_in_bytes"))
        .unwrap_or(0);

    let json = format!(
        "{{\"mem_current_mb\":{:.2},\"mem_limit_mb\":{:.2}}}",
        (mem_current as f64) / 1024.0 / 1024.0,
        (mem_limit as f64) / 1024.0 / 1024.0
    );

    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        json.len()
    );

    send_response(stream, &headers, Some(json.as_bytes()));
}

fn read_cgroup_metric(path: &str) -> Result<u64, io::Error> {
    let content = fs::read_to_string(path)?;
    let val = content.trim().parse::<u64>().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(val)
}
