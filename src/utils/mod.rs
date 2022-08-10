use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::ContentType;
use rocket::http::Header;
use rocket::{Request, Response};

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

#[derive(Responder)]
pub struct ImageResponse {
    pub inner: Vec<u8>,
    pub header: ContentType,
}

pub fn get_path_without_ext(path: &str) -> &str {
    match path.rsplit_once('.') {
        Some((new_path, _)) => new_path,
        None => path,
    }
}

pub fn get_ext_from_path(path: &str) -> Option<&str> {
    match path.rsplit_once('.') {
        Some((_, ext)) => Some(ext),
        None => None,
    }
}
