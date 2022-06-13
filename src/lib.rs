//! # link-rs
//! Generate hash-id based URLs and QR codes for use in URL shortening services.
//!
//! Basic example:
//! ```rs
//! mod link-rs::LinkGenerator
//!
//! fn main() {
//!     let mut link_gen = LinkGenerator::new("/some/redirect", 10);
//!     
//!     let link = link_gen::generate_url()
//!     println!("{:?}",link) // Link { key: "vq5ejng0p6", url: "/some/redirect/vq5ejng0p6" }
//! }
//! ```
//!
//! Complex example of an actix-web server using the `qrcode` feature:
//! ```rust
//! use actix_web::{
//!     get, http::header::ContentType, post, web, App, HttpResponse, HttpServer, Responder,
//! };
//!
//! use link_rs::LinkGenerator;
//! use qrcode::render::svg;
//! use serde::{Deserialize, Serialize};
//! use std::{
//!     collections::HashMap,
//!     sync::{Arc, Mutex},
//! };
//!
//! #[derive(Clone)]
//! struct AppState {
//!     data: Arc<Mutex<HashMap<String, String>>>,
//!     generator: Arc<Mutex<LinkGenerator>>,
//! }
//!
//! impl AppState {
//!     pub fn get_url(&self, url: String) -> Option<String> {
//!         self.data.lock().unwrap().get(&url).map(|u| u.to_owned())
//!     }
//!
//!     pub fn with_lock<F, T>(&self, func: F) -> T
//!     where
//!         F: FnOnce(&mut HashMap<String, String>, &mut LinkGenerator) -> T,
//!     {
//!         let mut map_lock = self.data.lock().unwrap();
//!         let mut gen_lock = self.generator.lock().unwrap();
//!         func(&mut *map_lock, &mut *gen_lock)
//!     }
//! }
//! #[derive(Deserialize)]
//! struct Request {
//!     url: String,
//! }
//!
//! #[derive(Serialize)]
//! struct Response {
//!     image: String,
//!     url: String,
//! }
//!
//! #[post("/")]
//! fn generate(req: web::Json<Request>, data: web::Data<AppState>) -> impl Responder {
//!     if let Ok((qr, url)) = data.with_lock(|map, gen| -> Result<(QrCode, String), QrError> {
//!         let (qr, link) = gen.generate_qr()?;
//!         map.insert(link.key, (*req.url).to_string());
//!         Ok((qr, link.url))
//!     }) {
//!         let image = qr.render::<svg::Color>().min_dimensions(200, 200).build();
//!
//!         HttpResponse::Ok()
//!             .content_type(ContentType::json())
//!             .body(serde_json::to_string(&Response { image, url }).unwrap())
//!     } else {
//!         HttpResponse::InternalServerError().finish()
//!     }
//! }
//!
//! #[get("/{url}")]
//! async fn redirect(url: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
//!     match state.get_url(url.to_string()) {
//!         Some(link) => HttpResponse::TemporaryRedirect()
//!             .append_header(("location", link))
//!             .finish(),
//!         None => HttpResponse::NotFound().finish(),
//!     }
//! }
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| App::new().service(redirect).service(generate))
//!         .bind(("127.0.0.1", 8080))?
//!         .run()
//!         .await
//! }
//!
//! ```
//!

use harsh::{Harsh, HarshBuilder};
use std::num::Wrapping;

#[cfg(feature = "qrcode")]
use qrcode::{types::QrError, QrCode};

#[derive(Debug, PartialEq, Eq)]
/// A generated URL and key value
pub struct Link {
    key: String,
    url: String,
}

impl Link {
    fn new(base: &str, key: String) -> Self {
        Link {
            url: [base, &key].concat(),
            key,
        }
    }
}
pub struct LinkGenerator {
    id: Wrapping<u64>,
    generator: Harsh,
    redirect_url: String,
}

impl LinkGenerator {
    /// Create a new LinkGenerator with salt to produce a non-standard generation pattern.
    pub fn new_with_salt(redirect_url: &str, length: usize, salt: &str) -> LinkGenerator {
        let harsh = HarshBuilder::new()
            .salt(salt)
            .alphabet("abcdefghijklmnopqrstuvwxyz0123456789")
            .length(length)
            .build()
            .unwrap();

        LinkGenerator {
            id: Wrapping(0),
            generator: harsh,
            redirect_url: [
                redirect_url,
                if !redirect_url.ends_with('/') {
                    "/"
                } else {
                    ""
                },
            ]
            .concat(),
        }
    }

    /// Create a new LinkGenerator.
    pub fn new(redirect_url: &str, length: usize) -> LinkGenerator {
        Self::new_with_salt(redirect_url, length, "")
    }

    /// Generate a new URL.
    pub fn generate_url(&mut self) -> Link {
        let hashed = self.generator.encode(&[self.id.0]);
        self.id += 1;
        Link::new(&self.redirect_url, hashed)
    }

    #[cfg(feature = "qrcode")]
    /// Generate a new URL and QR code.
    pub fn generate_qr(&mut self) -> Result<(QrCode, Link), QrError> {
        let link = self.generate_url();
        Ok((QrCode::new(&link.url)?, link))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "qrcode")]
    use qrcode::types::QrError;

    #[test]
    fn generate_link() {
        let mut s = LinkGenerator::new("/redirect", 10);
        let l = s.generate_url();

        println!("{:?}", l);

        assert_eq!(
            l,
            Link {
                key: "vq5ejng0p6".into(),
                url: "/redirect/vq5ejng0p6".into()
            }
        );
    }

    #[test]
    fn generate_link_with_salt() {
        let mut s = LinkGenerator::new_with_salt("/redirect", 10, "salt");
        let l = s.generate_url();

        assert_eq!(
            l,
            Link {
                key: "9x5eo4n7ow".into(),
                url: "/redirect/9x5eo4n7ow".into()
            }
        );
    }

    #[cfg(feature = "qrcode")]
    #[test]
    fn generate_qr() -> Result<(), QrError> {
        let mut s = LinkGenerator::new("/redirect", 10);
        let qr = s.generate_qr();

        assert!(qr.is_ok());
        Ok(())
    }
}
