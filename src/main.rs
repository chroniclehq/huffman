#[macro_use]
extern crate rocket;

use rocket::tokio::fs::File;
use rocket::tokio::io::AsyncReadExt;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;

mod services;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Responder)]
#[response(content_type = "image/webp")]
struct ImageResponse(Vec<u8>);

#[get("/library/<file..>")]
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
        .mount("/", routes![index])
        .mount("/", routes![optimize])
}
