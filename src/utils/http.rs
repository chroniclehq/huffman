use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::ContentType;
use rocket::http::Header;
use rocket::response::{Responder, Response};
use rocket::Request;

// Fairing for setting CORS Headers
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    // https://stackoverflow.com/a/64904947
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }
    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "GET, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

// Enum for controlling response cache. Header values based on recommendations in
// https://imagekit.io/blog/ultimate-guide-to-http-caching-for-static-assets
pub enum CacheControl {
    NoCache,
    Default,
}
impl<'h> Into<Header<'h>> for CacheControl {
    fn into(self) -> Header<'h> {
        match self {
            CacheControl::NoCache => Header::new("Cache-Control", "no-cache, no-store"),
            CacheControl::Default => Header::new("Cache-Control", "public, max-age=2629746"), // 1 month
        }
    }
}

// Responder for images along with content-type and cache-control header arguments
#[derive(Responder)]
pub struct ImageResponse {
    pub inner: Vec<u8>,
    pub content_type: ContentType,
    pub cache: CacheControl,
}
impl ImageResponse {
    pub fn new(value: Vec<u8>, content_type: ContentType, cache: CacheControl) -> Self {
        ImageResponse {
            inner: value,
            content_type,
            cache,
        }
    }
}

// Responder for strings with no-cache,no-store headers
#[derive(Responder)]
pub struct TextResponse {
    pub inner: &'static str,
    pub cache: CacheControl,
}
impl TextResponse {
    pub fn new(value: &'static str) -> Self {
        TextResponse {
            inner: value,
            cache: CacheControl::NoCache,
        }
    }
}
