use reqwest::{Client, Method};
use std::time::Instant;

pub struct RequestResult {
    pub status: u16,
    pub time_ms: u128,
    pub body: String,
}

pub async fn make_request(method_str: &str, url: &str) -> Result<RequestResult, String> {
    let client = Client::new();
    
    let method = match method_str.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        _ => return Err(format!("Método HTTP inválido: {}", method_str)),
    };

    let start_time = Instant::now();
    
    let response = client
        .request(method, url)
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

fn main() {
    println!("Requestman - Motor de rede preparado!");
}
