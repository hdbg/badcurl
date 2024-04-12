use std::time::Duration;

use badcurl::easy::{Easy, SslOpt};

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
        }
    };
}

fn handle() -> Easy {
    let mut e = Easy::new();
    t!(e.timeout(Duration::new(20, 0)));
    e
}


#[test]
fn fingerprint_chrome() {
    let mut handle = handle();

    let mut opt = SslOpt::new();
    opt.native_ca(true);

    t!(handle.url("https://example.com"));
    t!(handle.ssl_options(&opt));
    t!(handle.perform());
}