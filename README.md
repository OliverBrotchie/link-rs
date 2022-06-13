<div align="center">

  <h1><code>link-rs</code></h1>

  <strong>Generate hash-id based URLs and QR codes for use in URL shortening services.</strong>

</div>

## About

A simple implementation of a URL generation module. Allows for qr-code generation using the [`qrcode`](https://crates.io/crates/qrcode) library.

## 🔋 Usage

Basic Example:

```rs
mod link-rs::LinkGenerator


fn main() {
    let mut link_gen = LinkGenerator::new("/some/redirect", 10);

    let link = link_gen::generate_url()
    println!("{:?}",link) // Link { key: "vq5ejng0p6", url: "/some/redirect/vq5ejng0p6" }
}

```

Complex example of an actix-web server using the `qrcode` feature:
```rust
use actix_web::{
    get, http::header::ContentType, post, web, App, HttpResponse, HttpServer, Responder,
};
use link_rs::LinkGenerator;
use qrcode::render::svg;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct AppState {
    data: Arc<Mutex<HashMap<String, String>>>,
    generator: Arc<Mutex<LinkGenerator>>,
}

impl AppState {
    pub fn get_url(&self, url: String) -> Option<String> {
        self.data.lock().unwrap().get(&url).map(|u| u.to_owned())
    }

    pub fn with_lock<F, T>(&self, func: F) -> T
    where
        F: FnOnce(&mut HashMap<String, String>, &mut LinkGenerator) -> T,
    {
        let mut map_lock = self.data.lock().unwrap();
        let mut gen_lock = self.generator.lock().unwrap();
        func(&mut *map_lock, &mut *gen_lock)
    }
}

#[derive(Deserialize)]
struct Request {
    url: String,
}

#[derive(Serialize)]
struct Response {
    image: String,
    url: String,
}

#[post("/")]
fn generate(req: web::Json<Request>, data: web::Data<AppState>) -> impl Responder {
    if let Ok((qr, url)) = data.with_lock(|map, gen| -> Result<(QrCode, String), QrError> {
        let (qr, link) = gen.generate_qr()?;

        map.insert(link.key, (*req.url).to_string());
        Ok((qr, link.url))
    }) {
        let image = qr.render::<svg::Color>().min_dimensions(200, 200).build();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&Response { image, url }).unwrap())
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[get("/{url}")]
async fn redirect(url: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    match state.get_url(url.to_string()) {
        Some(link) => HttpResponse::TemporaryRedirect()
            .append_header(("location", link))
            .finish(),
        None => HttpResponse::NotFound().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(redirect).service(generate))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

```

### 🛠️ Add via `cargo.toml`

```toml
link-rs = { version = "0.1.0", features = ["qrcode"] }
```

### 🔬 Test using `cargo test`

```sh
cargo test --features qrcode
```
