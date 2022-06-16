//! # link-rs
//! Generate hash-id based URLs and QR codes for use in URL shortening services.
//!
//! Basic example:
//! ```rs
//! mod link_rs::LinkGenerator
//!
//! fn main() {
//!     let mut link_gen = LinkGenerator::new("/some/redirect", 10);
//!     
//!     let link = link_gen::generate_url()
//!     println!("{:?}",link) // Link { key: "vq5ejng0p6", url: "/some/redirect/vq5ejng0p6" }
//! }
//! ```
//!
//! To see a complete implementation of a url shortening service,
//! please take a look at the [`actix-web`](https://github.com/OliverBrotchie/link-rs/blob/main/examples/actix.rs) example.

use harsh::{Harsh, HarshBuilder};
use std::num::Wrapping;

#[cfg(feature = "qrcode")]
use qrcode::{types::QrError, QrCode};

#[derive(Debug, PartialEq, Eq)]
/// A generated URL and key value
pub struct Link {
    pub key: String,
    pub url: String,
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
    pub redirect_url: String,
}

impl LinkGenerator {
    /// Create a new LinkGenerator with salt to produce a non-predictable generation pattern.
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

    /// Create a new LinkGenerator with a pre-set internal ID used to generate the hash for the next URL.
    pub fn new_with_internal_id(
        id: u64,
        redirect_url: &str,
        length: usize,
        salt: Option<&str>,
    ) -> LinkGenerator {
        let mut gen = Self::new_with_salt(redirect_url, length, salt.unwrap_or(""));
        gen.set_internal_id(id);
        gen
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

    /// Get the current value of the internal ID used to generate the hash for the next URL.
    pub fn get_internal_id(&self) -> u64 {
        self.id.0
    }

    /// Set the current value of the internal ID used to generate the hash for the next URL.
    pub fn set_internal_id(&mut self, input: u64) {
        self.id.0 = input
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

    #[test]
    fn generate_qr() -> Result<(), QrError> {
        let mut s = LinkGenerator::new("/redirect", 10);
        let qr = s.generate_qr();

        assert!(qr.is_ok());
        Ok(())
    }
}
