use actix_web::{get, http, post, web, App, HttpResponse, HttpServer, Responder};
use link_rs::LinkGenerator;
use qrcode::{render::svg, types::QrError, QrCode};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct StateGaurd {
    state: Arc<Mutex<State>>,
}

struct State {
    data: HashMap<String, String>,
    generator: LinkGenerator,
}

impl StateGaurd {
    fn new() -> Self {
        let state = Arc::new(Mutex::new(State {
            data: HashMap::new(),
            generator: LinkGenerator::new("/redirect", 10),
        }));
        StateGaurd { state }
    }

    pub fn get_url(&self, url: String) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .data
            .get(&url)
            .map(|u| u.to_owned())
    }

    pub fn with_lock<F, T>(&self, func: F) -> T
    where
        F: FnOnce(&mut State) -> T,
    {
        let mut lock = self.state.lock().unwrap();
        func(&mut *lock)
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

#[post("/generate")]
async fn generate(req: web::Json<Request>, data: web::Data<StateGaurd>) -> impl Responder {
    match data.with_lock(|state| -> Result<(QrCode, String), QrError> {
        let (qr, link) = state.generator.generate_qr()?;

        state.data.insert(link.key, (*req.url).to_string());
        Ok((qr, link.url))
    }) {
        Ok((qr, url)) => {
            let image = qr.render::<svg::Color>().min_dimensions(200, 200).build();

            println!("Generated new redirect: {}", url);

            HttpResponse::Ok()
                .content_type(http::header::ContentType::json())
                .body(serde_json::to_string(&Response { image, url }).unwrap())
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/redirect/{url}")]
async fn redirect(url: web::Path<String>, state: web::Data<StateGaurd>) -> impl Responder {
    println!("Redirect on: {url}");

    match state.get_url(url.to_string()) {
        Some(link) => HttpResponse::TemporaryRedirect()
            .append_header(("location", link))
            .finish(),
        None => HttpResponse::NotFound().finish(),
    }
}

const PORT: u16 = 8080;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting actix URL shortening server at: http://localhost:{PORT} 🚀");
    println!("Try posting a URL to `/generate` and then visit `/redirect/<generated_key>`");

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(StateGaurd::new()))
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .service(redirect)
            .service(generate)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("127.0.0.1", PORT))?
    .run()
    .await
}
