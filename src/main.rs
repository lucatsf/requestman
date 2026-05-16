slint::include_modules!();

use reqwest::{Client, Method, header::{HeaderMap, HeaderName, HeaderValue}};
use std::time::Instant;

pub struct RequestResult {
    pub status: u16,
    pub time_ms: u128,
    pub body: String,
    pub size_bytes: usize,
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
    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Erro ao ler a resposta: {}", e))?;
    
    let size_bytes = body_bytes.len();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
        
    let time_ms = start_time.elapsed().as_millis();

    Ok(RequestResult {
        status,
        time_ms,
        body,
        size_bytes,
    })
}

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    
    let ui_handle = ui.as_weak();

    ui.on_send_request(move || {
        let ui = ui_handle.unwrap();
        
        let url = ui.get_url().to_string();
        let method = ui.get_method().to_string();
        let body = ui.get_request_body().to_string();
        
        let h1_k = ui.get_header1_key().to_string();
        let h1_v = ui.get_header1_val().to_string();
        let h2_k = ui.get_header2_key().to_string();
        let h2_v = ui.get_header2_val().to_string();

        let mut headers = Vec::new();
        if !h1_k.is_empty() { headers.push((h1_k, h1_v)); }
        if !h2_k.is_empty() { headers.push((h2_k, h2_v)); }

        // Set loading state
        ui.set_response_body("Enviando requisição...".into());
        ui.set_response_status("Carregando".into());
        ui.set_response_time("0 ms".into());
        ui.set_response_size("0 B".into());

        let ui_handle_async = ui.as_weak();

        tokio::spawn(async move {
            let result = make_request(&method, &url, headers, &body).await;

            let _ = slint::invoke_from_event_loop(move || {
                let ui = ui_handle_async.unwrap();
                match result {
                    Ok(res) => {
                        ui.set_response_body(res.body.into());
                        ui.set_response_status(format!("{} Status", res.status).into());
                        ui.set_response_time(format!("{} ms", res.time_ms).into());
                        ui.set_response_size(format!("{} B", res.size_bytes).into());
                    }
                    Err(e) => {
                        ui.set_response_body(e.into());
                        ui.set_response_status("Erro".into());
                    }
                }
            });
        });
    });

    println!("Requestman - Iniciando interface conectada!");
    ui.run()
}
