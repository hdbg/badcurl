//! Rust bindings to the libcurl-imperosnate C library
//! 
//! Fork of [curl](https://crates.io/crates/curl) crate with support for [libcurl-impersonate](https://github.com/yifeikong/curl-impersonate)
//!
//! This crate contains bindings for an HTTP/HTTPS client which is powered by
//! [libcurl], the same library behind the `curl` command line tool. The API
//! currently closely matches that of libcurl itself, except that a Rustic layer
//! of safety is applied on top.
//!
//! [libcurl]: https://curl.haxx.se/libcurl/
//!
//! # The "Easy" API
//!
//! The easiest way to send a request is to use the `Easy` api which corresponds
//! to `CURL` in libcurl. This handle supports a wide variety of options and can
//! be used to make a single blocking request in a thread. Callbacks can be
//! specified to deal with data as it arrives and a handle can be reused to
//! cache connections and such.
//!
//! ```rust,no_run
//! use std::io::{stdout, Write};
//!
//! use badcurl::easy::Easy;
//!
//! // Write the contents of rust-lang.org to stdout
//! let mut easy = Easy::new();
//! easy.url("https://www.rust-lang.org/").unwrap();
//! easy.write_function(|data| {
//!     stdout().write_all(data).unwrap();
//!     Ok(data.len())
//! }).unwrap();
//! easy.perform().unwrap();
//! ```
//!
//! # What about multiple concurrent HTTP requests?
//!
//! One option you have currently is to send multiple requests in multiple
//! threads, but otherwise libcurl has a "multi" interface for doing this
//! operation. Initial bindings of this interface can be found in the `multi`
//! module, but feedback is welcome!
//!
//! # Where does libcurl come from?
//!
//! This crate links to the `curl-sys` crate which is in turn responsible for
//! acquiring and linking to the libcurl library. Currently this crate will
//! build libcurl from source if one is not already detected on the system.
//!
//! There is a large number of releases for libcurl, all with different sets of
//! capabilities. Robust programs may wish to inspect `Version::get()` to test
//! what features are implemented in the linked build of libcurl at runtime.
//!
//! # Initialization
//!
//! The underlying libcurl library must be initialized before use and has
//! certain requirements on how this is done. Check the documentation for
//! [`init`] for more details.
//! 
//! # Impersonation
//! This crate supports impersonationg of different browsers. You can see all supported browsers and their versions in 
//! [`Profile`](easy::Profile) enum.
//! 
//! ```rust
//! use std::io::{stdout, Write};
//!
//! use badcurl::easy::{Easy, Profile};
//!
//! // Write the contents of rust-lang.org to stdout
//! let mut easy = Easy::new();
//! easy.url("https://www.rust-lang.org/").unwrap();
//! 
//! easy.impersonate(Profile::Chrome120).unwrap();
//! 
//! easy.write_function(|data| {
//!     stdout().write_all(data).unwrap();
//!     Ok(data.len())
//! }).unwrap();
//! 
//! easy.perform().unwrap();
//! ```

#![deny(missing_docs, missing_debug_implementations)]

use std::ffi::CStr;
use std::str;
use std::sync::Once;

pub use crate::error::{Error, FormError, MultiError, ShareError};
mod error;

pub use crate::version::{Protocols, Version};
mod version;

pub mod easy;
pub mod multi;
mod panic;

#[cfg(test)]
static INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Initializes the underlying libcurl library.
///
/// The underlying libcurl library must be initialized before use, and must be
/// done so on the main thread before any other threads are created by the
/// program. This crate will do this for you automatically in the following
/// scenarios:
///
/// - Creating a new [`Easy`][easy::Easy] or [`Multi`][multi::Multi] handle
/// - At program startup on Windows, macOS, Linux, Android, or FreeBSD systems
///
/// This should be sufficient for most applications and scenarios, but in any
/// other case, it is strongly recommended that you call this function manually
/// as soon as your program starts.
///
/// Calling this function more than once is harmless and has no effect.
#[inline]
pub fn init() {
    /// Used to prevent concurrent or duplicate initialization.
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        #[cfg(need_openssl_init)]
        openssl_probe::init_ssl_cert_env_vars();
        #[cfg(need_openssl_init)]
        openssl_sys::init();

        unsafe {
            assert_eq!(badcurl_sys::curl_global_init(badcurl_sys::CURL_GLOBAL_ALL), 0);
        }

        #[cfg(test)]
        {
            INITIALIZED.store(true, std::sync::atomic::Ordering::SeqCst);
        }

        // Note that we explicitly don't schedule a call to
        // `curl_global_cleanup`. The documentation for that function says
        //
        // > You must not call it when any other thread in the program (i.e. a
        // > thread sharing the same memory) is running. This doesn't just mean
        // > no other thread that is using libcurl.
        //
        // We can't ever be sure of that, so unfortunately we can't call the
        // function.
    });
}

/// An exported constructor function. On supported platforms, this will be
/// invoked automatically before the program's `main` is called. This is done
/// for the convenience of library users since otherwise the thread-safety rules
/// around initialization can be difficult to fulfill.
///
/// This is a hidden public item to ensure the symbol isn't optimized away by a
/// rustc/LLVM bug: https://github.com/rust-lang/rust/issues/47384. As long as
/// any item in this module is used by the final binary (which `init` will be)
/// then this symbol should be preserved.
#[used]
#[doc(hidden)]
#[cfg_attr(
    any(target_os = "linux", target_os = "freebsd", target_os = "android"),
    link_section = ".init_array"
)]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
pub static INIT_CTOR: extern "C" fn() = {
    /// This is the body of our constructor function.
    #[cfg_attr(
        any(target_os = "linux", target_os = "android"),
        link_section = ".text.startup"
    )]
    extern "C" fn init_ctor() {
        init();
    }

    init_ctor
};

unsafe fn opt_str<'a>(ptr: *const libc::c_char) -> Option<&'a str> {
    if ptr.is_null() {
        None
    } else {
        Some(str::from_utf8(CStr::from_ptr(ptr).to_bytes()).unwrap())
    }
}

fn cvt(r: badcurl_sys::CURLcode) -> Result<(), Error> {
    if r == badcurl_sys::CURLE_OK {
        Ok(())
    } else {
        Err(Error::new(r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "android"
    ))]
    fn is_initialized_before_main() {
        assert!(INITIALIZED.load(std::sync::atomic::Ordering::SeqCst));
    }
}
