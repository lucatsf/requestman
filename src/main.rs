slint::include_modules!();

use reqwest::{Client, Method, header::{HeaderMap, HeaderName, HeaderValue}};
use std::time::Instant;

pub struct RequestResult {
    pub status: u16,
    pub time_ms: u128,
    pub body: String,
}

pub async fn make_request(
    method_str: &str, 
    url: &str,
    headers_vec: Vec<(String, String)>,
    request_body: &str,
) -> Result<RequestResult, String> {
    let client = Client::new();
    
    let method = match method_str.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        _ => return Err(format!("Método HTTP inválido: {}", method_str)),
    };

    let mut header_map = HeaderMap::new();
    for (key, value) in headers_vec {
        if !key.trim().is_empty() {
            if let (Ok(k), Ok(v)) = (HeaderName::from_bytes(key.as_bytes()), HeaderValue::from_str(&value)) {
                header_map.insert(k, v);
            }
        }
    }

    let start_time = Instant::now();
    
    let mut request_builder = client.request(method, url).headers(header_map);
    
    if !request_body.trim().is_empty() {
        request_builder = request_builder.body(request_body.to_string());
    }

    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("Erro na requisição: {}", e))?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Erro ao ler a resposta: {}", e))?;
        
    let time_ms = start_time.elapsed().as_millis();

    Ok(RequestResult {
        status,
        time_ms,
        body,
    })
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    println!("Requestman - Iniciando interface (Apenas visual, sem lógica ainda)!");
    ui.run()
}
