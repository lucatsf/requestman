slint::include_modules!();

mod collection;

use collection::{Collection, SavedRequest, HeaderPair};
use reqwest::{Client, Method, header::{HeaderMap, HeaderName, HeaderValue}};
use slint::{ModelRc, VecModel};
use std::rc::Rc;
use std::cell::RefCell;
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
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    
    // Tenta formatar como JSON, senão mantém a string original
    let body = match serde_json::from_str::<serde_json::Value>(&body_str) {
        Ok(json) => serde_json::to_string_pretty(&json).unwrap_or(body_str),
        Err(_) => body_str,
    };
        
    let time_ms = start_time.elapsed().as_millis();

    Ok(RequestResult {
        status,
        time_ms,
        body,
        size_bytes,
    })
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    
    let collection_path = "requestman_collection.json";
    let current_collection = collection::load_collection(collection_path);
    let shared_collection = Rc::new(RefCell::new(current_collection));
    
    let update_ui_collection = |ui: &MainWindow, col: &Collection| {
        let items: Vec<CollectionItem> = col.requests.iter().map(|req| {
            CollectionItem {
                name: req.name.clone().into(),
                method: req.method.clone().into(),
            }
        }).collect();
        let model = Rc::new(VecModel::from(items));
        ui.set_collection_items(ModelRc::from(model));
    };
    
    update_ui_collection(&ui, &shared_collection.borrow());

    let ui_handle_send = ui.as_weak();
    ui.on_send_request(move || {
        let ui = ui_handle_send.unwrap();
        
        let mut url = ui.get_url().to_string();
        let method = ui.get_method().to_string();
        let body = ui.get_request_body().to_string();
        
        // Processa os Query Parameters
        let p1_k = ui.get_param1_key().to_string();
        let p1_v = ui.get_param1_val().to_string();
        let p2_k = ui.get_param2_key().to_string();
        let p2_v = ui.get_param2_val().to_string();

        let mut query_params = Vec::new();
        if !p1_k.is_empty() { query_params.push(format!("{}={}", p1_k, p1_v)); }
        if !p2_k.is_empty() { query_params.push(format!("{}={}", p2_k, p2_v)); }

        if !query_params.is_empty() {
            let query_str = query_params.join("&");
            if url.contains('?') {
                url.push_str("&");
            } else {
                url.push_str("?");
            }
            url.push_str(&query_str);
            // Atualiza a UI para o usuário ver a URL construída
            ui.set_url(url.clone().into());
        }

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

        // Para evitar travar a UI, disparamos a requisição em uma thread separada
        std::thread::spawn(move || {
            // Criamos um runtime isolado apenas para a requisição
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                rt.block_on(async move {
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
            }
        });
    });

    let ui_save = ui.as_weak();
    let col_save = shared_collection.clone();
    ui.on_save_request(move |name| {
        let ui = ui_save.unwrap();
        let mut req = SavedRequest {
            name: name.to_string(),
            url: ui.get_url().to_string(),
            method: ui.get_method().to_string(),
            body: ui.get_request_body().to_string(),
            headers: vec![],
        };

        let h1_k = ui.get_header1_key().to_string();
        let h1_v = ui.get_header1_val().to_string();
        let h2_k = ui.get_header2_key().to_string();
        let h2_v = ui.get_header2_val().to_string();

        if !h1_k.is_empty() { req.headers.push(HeaderPair { key: h1_k, value: h1_v }); }
        if !h2_k.is_empty() { req.headers.push(HeaderPair { key: h2_k, value: h2_v }); }

        let mut col = col_save.borrow_mut();
        if let Some(existing) = col.requests.iter_mut().find(|r| r.name == req.name) {
            *existing = req;
        } else {
            col.requests.push(req);
        }
        
        let _ = collection::save_collection("requestman_collection.json", &col);
        
        let items: Vec<CollectionItem> = col.requests.iter().map(|r| {
            CollectionItem { name: r.name.clone().into(), method: r.method.clone().into() }
        }).collect();
        ui.set_collection_items(ModelRc::from(Rc::new(VecModel::from(items))));
    });

    let ui_load = ui.as_weak();
    let col_load = shared_collection.clone();
    ui.on_load_request(move |index| {
        let ui = ui_load.unwrap();
        let col = col_load.borrow();
        if let Some(req) = col.requests.get(index as usize) {
            ui.set_url(req.url.clone().into());
            ui.set_method(req.method.clone().into());
            ui.set_request_body(req.body.clone().into());
            
            // reset headers
            ui.set_header1_key("".into()); ui.set_header1_val("".into());
            ui.set_header2_key("".into()); ui.set_header2_val("".into());

            if let Some(h) = req.headers.get(0) {
                ui.set_header1_key(h.key.clone().into());
                ui.set_header1_val(h.value.clone().into());
            }
            if let Some(h) = req.headers.get(1) {
                ui.set_header2_key(h.key.clone().into());
                ui.set_header2_val(h.value.clone().into());
            }
        }
    });

    println!("Requestman - Iniciando interface conectada!");
    ui.run()
}
