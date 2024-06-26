//! Simple HTTPS GET
//!
//! This example is a Rust adaptation of the [C example of the same
//! name](https://curl.se/libcurl/c/https.html).

extern crate badcurl;

use badcurl::easy::Easy;
use std::io::{stdout, Write};

fn main() -> Result<(), badcurl::Error> {
    let mut curl = Easy::new();

    curl.url("https://example.com/")?;
    curl.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    })?;

    curl.perform()?;

    Ok(())
}
