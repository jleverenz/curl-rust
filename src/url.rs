//! Url - URL parsing functionality

use libc::c_char;
use std::ffi::{CStr, CString};
use std::str;

use curl_sys::{
    CURLUPART_URL,
    CURLUPART_SCHEME,
    CURLUPART_HOST,
    CURLUPART_PORT,
    CURLUPART_PATH,
    CURLUPART_QUERY,
    CURL_URL,
    CURLUPARTcode
};

/// Wrapper to underlying cURL URL parsing interface.
#[derive(Debug)]
pub struct UrlApi {
    handle: *mut curl_sys::CURL_URL,
}

/// Parse URL returned by wrapper.
#[derive(Debug)]
pub struct Url {
    /// URL scheme part
    pub scheme: String,
    /// URL host part
    pub host: String,
    /// URL port part
    pub port: String,
    /// URL path part
    pub path: String,
    /// USR query part
    pub query: String,
}

impl UrlApi {

    /// Allocate new handle to URL API
    pub fn new() -> UrlApi {
        crate::init();
        unsafe {
            let handle = curl_sys::curl_url();
            assert!(!handle.is_null());
            UrlApi { handle }
        }
    }

    /// Set the URL to parse
    pub fn url_set(self: &Self, url: &str) {
        let u = CString::new(url).expect("url_set error");

        unsafe {
            let rc = curl_sys::curl_url_set(self.handle, CURLUPART_URL, u.as_ptr(), 0);
            assert!(rc == 0, "curl_url_set failed {}", rc);
        }
    }

    /// Get the parsed URL
    pub fn get_parts(self: &Self) -> Url {
        Url {
            scheme: get_url_part(self.handle, CURLUPART_SCHEME),
            host: get_url_part(self.handle, CURLUPART_HOST),
            port: get_url_part(self.handle, CURLUPART_PORT),
            path: get_url_part(self.handle, CURLUPART_PATH),
            query: get_url_part(self.handle, CURLUPART_QUERY),
        }
    }
}

impl Drop for UrlApi {
    fn drop(&mut self) {
        unsafe {
            curl_sys::curl_url_cleanup(self.handle);
        }
    }
}

fn get_url_part(handle: *mut CURL_URL, part: CURLUPARTcode) -> String {
    unsafe {
        let mut p: *const c_char = std::ptr::null_mut();
        let rc = curl_sys::curl_url_get(handle, part, &mut p, 0);
        assert!(rc == 0, "curl_url_get failed {}", rc);
        let ret = str::from_utf8(CStr::from_ptr(p).to_bytes()).unwrap();
        let ret = String::from(ret);
        curl_sys::curl_free(p as *mut _);
        return ret;
    }
}

/// Convience method to parse a url str
pub fn parse_url(url: &str) -> Url {
    let url_api = UrlApi::new();
    url_api.url_set(url);
    url_api.get_parts()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_api_smoke() {
        let url_api = UrlApi::new();
        url_api.url_set("https://example.com:449/foo/bar?name=moo");

        let url = url_api.get_parts();

        assert_eq!("https", url.scheme);
        assert_eq!("example.com", url.host);
        assert_eq!("449", url.port);
        assert_eq!("/foo/bar", url.path);
        assert_eq!("name=moo", url.query);
    }

    #[test]
    fn parse_url_smoke() {
        let url = parse_url("https://example.com:449/foo/bar?name=moo");

        assert_eq!("https", url.scheme);
        assert_eq!("example.com", url.host);
        assert_eq!("449", url.port);
        assert_eq!("/foo/bar", url.path);
        assert_eq!("name=moo", url.query);
    }
}
