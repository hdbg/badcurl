use std::time::Duration;

use badcurl::easy::Easy;

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
    t!(handle.url("https://example.com"));
    t!(handle.impersonate(badcurl::easy::Profile::Chrome100, true));
    t!(handle.perform());
}