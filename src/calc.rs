use std::net::TcpStream;
use std::sync::Arc;
use crate::state::AppState;
use crate::handler::send_response;

pub fn handle_calculate(stream: &mut TcpStream, query: &str, state: Arc<AppState>) {
    let mut basket: f64 = 250.0;
    let mut distance: f64 = 3.5;

    for param in query.split('&') {
        let mut split = param.split('=');
        if let (Some(key), Some(val)) = (split.next(), split.next()) {
            match key {
                "basket" => {
                    if let Ok(v) = val.parse::<f64>() {
                        basket = v;
                    }
                }
                "distance" => {
                    if let Ok(v) = val.parse::<f64>() {
                        distance = v;
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
