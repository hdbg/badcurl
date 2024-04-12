#[cfg(all(feature = "static-badcurl", not(feature = "protocol-ftp")))]
#[test]
fn static_with_ftp_disabled() {
    assert!(badbadcurl::Version::get()
        .protocols()
        .filter(|&p| p == "ftp")
        .next()
        .is_none());
}

#[cfg(all(feature = "static-badcurl", feature = "protocol-ftp"))]
#[test]
fn static_with_ftp_enabled() {
    assert!(badbadcurl::Version::get()
        .protocols()
        .filter(|&p| p == "ftp")
        .next()
        .is_some());
}
