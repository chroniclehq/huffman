#[macro_use]
extern crate rocket;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::tokio::fs::File;
use rocket::tokio::io::AsyncReadExt;
use rocket::{Request, Response};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;

mod services;

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

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Responder)]
#[response(content_type = "image/webp")]
struct ImageResponse(Vec<u8>);

#[get("/test/<file..>")]
async fn optimize(file: PathBuf) -> Option<ImageResponse> {
    let path: PathBuf =
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets/test/")).join(file);

    println!("{}", path.as_os_str().to_string_lossy());

    let res: Result<File, Error> = File::open(path).await;

    match res {
        Ok(file) => {
            let mut file: File = file.try_clone().await.unwrap();
            let mut original_image: Vec<u8> = Vec::new();

            let _size = file.read_to_end(&mut original_image).await.unwrap();
            let time = Instant::now();

            let result: Result<Vec<u8>, libvips::error::Error> =
                services::image::process_image(&original_image);
            println!("Optimized in {:.2?}", time.elapsed());

            match result {
                Ok(optimised_image) => {
                    println!("Optimised");
                    Some(ImageResponse(optimised_image))
                }
                Err(error) => {
                    println!("{}", error);
                    Some(ImageResponse(original_image))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            None
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(CORS)
        .mount("/", routes![index])
        .mount("/", routes![optimize])
}
