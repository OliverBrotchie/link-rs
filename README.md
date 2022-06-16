<div align="center">

  <h1><code>link-rs</code></h1>

  <strong>Generate hash-id based URLs and QR codes for use in URL shortening services.</strong>

</div>

## 🔍 About

A simple implementation of a URL generation module. Allows for qr-code generation using the [`qrcode`](https://crates.io/crates/qrcode) library.

## 🔋 Usage

Basic Example:

```rust
mod link_rs::LinkGenerator


fn main() {
    let mut link_gen = LinkGenerator::new("/some/redirect", 10);

    let link = link_gen::generate_url()
    println!("{:?}", link) // Link { key: "vq5ejng0p6", url: "/some/redirect/vq5ejng0p6" }
}

```

To see a complete implementation of a url shortening service, please take a look at the [`actix-web`](https://github.com/OliverBrotchie/link-rs/blob/main/examples/actix.rs) example.

### 🛠️ Add via `cargo.toml`

```toml
link_rs = { version = "0.1.2", features = ["qrcode"] }
```

### 🔬 Test using `cargo test`

```sh
cargo test
```
