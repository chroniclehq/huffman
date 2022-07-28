#[macro_use]
extern crate rocket;
extern crate dotenv;

use dotenv::dotenv;
use rocket::http::Status;
use rocket::tokio::fs::File;
use rocket::tokio::io::AsyncReadExt;
use std::env;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;

mod drivers;
mod services;
mod utils;

#[get("/ping")]
fn index() -> &'static str {
    println!("Received ping");
    "pong!"
}

#[derive(Responder)]
#[response(content_type = "image/webp")]
struct ImageResponse(Vec<u8>);

#[get("/test/<file..>")]
async fn test(file: PathBuf) -> Option<ImageResponse> {
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

#[get("/library/<file..>")]
async fn generate(file: PathBuf) -> Status {
    let path: PathBuf =
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets/test/")).join(&file);

    println!("{}", path.as_os_str().to_string_lossy());

    let res: Result<File, Error> = File::open(path).await;

    match res {
        Ok(original_file) => {
            let mut original_file: File = original_file.try_clone().await.unwrap();
            let mut original_image: Vec<u8> = Vec::new();

            let size = original_file
                .read_to_end(&mut original_image)
                .await
                .unwrap();

            let result: Result<Vec<u8>, libvips::error::Error> =
                services::image::process_image(&original_image);

            match result {
                Ok(optimised_image) => {
                    println!("Optimised");

                    let (file_name_without_ext, _) =
                        file.to_str().unwrap().split_once('.').unwrap();
                    let dest_location = format!(
                        "{}/src/assets/test/updated/{}.webp",
                        env!("CARGO_MANIFEST_DIR"),
                        file_name_without_ext
                    );
                    let dest_path: &Path = Path::new(&dest_location);
                    fs::create_dir_all(dest_path.parent().unwrap()).unwrap();
                    fs::write(dest_location, &optimised_image).unwrap();
                    Status::Ok
                }
                Err(error) => {
                    println!("{}", error);
                    Status::InternalServerError
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            Status::InternalServerError
        }
    }
}

#[get("/library/<file..>")]
async fn fetch(file: PathBuf) -> Option<ImageResponse> {
    let (file_name_without_ext, _) = file.to_str().unwrap().split_once('.').unwrap();
    let optimised_image_location = format!(
        "{}/src/assets/test/updated/{}.webp",
        env!("CARGO_MANIFEST_DIR"),
        file_name_without_ext
    );
    let optimised_image_path: &Path = Path::new(&optimised_image_location);

    if optimised_image_path.exists() {
        let res: Result<File, Error> = File::open(optimised_image_path).await;

        match res {
            Ok(original_file) => {
                println!("file exists. Returning cached image");

                let mut original_image: Vec<u8> = Vec::new();
                original_file
                    .try_clone()
                    .await
                    .unwrap()
                    .read_to_end(&mut original_image)
                    .await
                    .unwrap();
                Some(ImageResponse(original_image))
            }
            Err(error) => {
                println!("Error while reading file from cache.  Error is {}", error);
                None
            }
        }
    } else {
        let path: PathBuf =
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets/test/")).join(&file);

        let res: Result<File, Error> = File::open(path).await;

        match res {
            Ok(original_file) => {
                let mut original_file: File = original_file.try_clone().await.unwrap();
                let mut original_image: Vec<u8> = Vec::new();

                let size = original_file
                    .read_to_end(&mut original_image)
                    .await
                    .unwrap();

                let result: Result<Vec<u8>, libvips::error::Error> =
                    services::image::process_image(&original_image);

                match result {
                    Ok(optimised_image) => {
                        println!("Optimised");

                        let (file_name_without_ext, _) =
                            file.to_str().unwrap().split_once('.').unwrap();
                        let dest_location = format!(
                            "{}/src/assets/test/updated/{}.webp",
                            env!("CARGO_MANIFEST_DIR"),
                            file_name_without_ext
                        );
                        let dest_path: &Path = Path::new(&dest_location);
                        fs::create_dir_all(dest_path.parent().unwrap()).unwrap();
                        fs::write(dest_location, &optimised_image).unwrap();
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
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build()
        .attach(utils::CORS)
        .mount("/", routes![index])
        .mount("/", routes![test])
        .mount("/generate", routes![generate])
        .mount("/fetch", routes![fetch])
}
