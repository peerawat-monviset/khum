use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::sync::Arc;
use crate::state::AppState;
use crate::calc::handle_calculate;
use crate::metrics::{handle_metrics, handle_sysinfo};

pub async fn handle_connection(mut stream: TcpStream, state: Arc<AppState>) {
    let mut request_bytes = Vec::new();
    let mut buffer = [0; 512];
    
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                request_bytes.extend_from_slice(&buffer[..n]);
                if request_bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
                if request_bytes.len() >= 8192 {
                    send_response(&mut stream, "HTTP/1.1 431 Request Header Fields Too Large\r\n\r\n", None).await;
                    return;
                }
            }
            Err(e) => {
                eprintln!("Failed to read stream: {}", e);
                return;
            }
        }
    }

    let request_str = match std::str::from_utf8(&request_bytes) {
        Ok(s) => s,
        Err(_) => return,
    };
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
        send_response(&mut stream, "HTTP/1.1 405 Method Not Allowed\r\n\r\n", None).await;
        return;
    }

    let (path, query) = if let Some(idx) = full_path.find('?') {
        (&full_path[..idx], &full_path[idx + 1..])
    } else {
        (full_path, "")
    };

    match path {
        "/" | "/index.html" => {
            serve_cached_file(&mut stream, &state.static_files["/index.html"], "text/html; charset=utf-8").await;
        }
        "/main.css" => {
            serve_cached_file(&mut stream, &state.static_files["/main.css"], "text/css; charset=utf-8").await;
        }
        "/main.js" => {
            serve_cached_file(&mut stream, &state.static_files["/main.js"], "application/javascript; charset=utf-8").await;
        }
        "/locales/en.json" => {
            serve_cached_file(&mut stream, &state.static_files["/locales/en.json"], "application/json; charset=utf-8").await;
        }
        "/locales/th.json" => {
            serve_cached_file(&mut stream, &state.static_files["/locales/th.json"], "application/json; charset=utf-8").await;
        }
        "/api/calculate" => {
            handle_calculate(&mut stream, query, state).await;
        }
        "/api/metrics" => {
            handle_metrics(&mut stream, state).await;
        }
        "/api/sysinfo" => {
            handle_sysinfo(&mut stream).await;
        }
        "/api/icon" => {
            handle_icon(&mut stream, query, state).await;
        }
        _ => {
            send_response(&mut stream, "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n", None).await;
        }
    }
}

pub async fn handle_icon(stream: &mut TcpStream, query: &str, state: Arc<AppState>) {
    let mut provider = "";
    for param in query.split('&') {
        let mut split = param.split('=');
        if let (Some(key), Some(val)) = (split.next(), split.next()) {
            if key == "provider" {
                provider = val;
            }
        }
    }

    let url_opt = {
        let icon_urls = state.icon_urls.read().unwrap();
        icon_urls.get(provider).cloned()
    };

    if let Some(url) = url_opt {
        let headers = format!(
            "HTTP/1.1 307 Temporary Redirect\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            url
        );
        let _ = stream.write_all(headers.as_bytes()).await;
    } else {
        // Fallback default SVG colors if not resolved yet
        let fallback_url = match provider {
            "grab" => "https://raw.githubusercontent.com/spothq/car-logos/master/logos-logos/grab.png", // or a placeholder
            _ => "",
        };
        if !fallback_url.is_empty() {
            let headers = format!(
                "HTTP/1.1 307 Temporary Redirect\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                fallback_url
            );
            let _ = stream.write_all(headers.as_bytes()).await;
        } else {
            send_response(stream, "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", None).await;
        }
    }
}

pub async fn serve_cached_file(stream: &mut TcpStream, contents: &[u8], content_type: &str) {
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        content_type,
        contents.len()
    );
    send_response(stream, &headers, Some(contents)).await;
}

pub async fn send_response(stream: &mut TcpStream, headers: &str, body: Option<&[u8]>) {
    if let Err(e) = stream.write_all(headers.as_bytes()).await {
        eprintln!("Failed to write headers: {}", e);
        return;
    }
    if let Some(contents) = body {
        if let Err(e) = stream.write_all(contents).await {
            eprintln!("Failed to write body: {}", e);
        }
    }
}
