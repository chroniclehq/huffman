#[macro_use]
extern crate rocket;
extern crate dotenv;

mod drivers;
mod services;
mod utils;

use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::tokio::task;
use rocket::State;
use services::storage::{Storage, UploadData};
use std::path::PathBuf;
use std::time::Instant;

#[get("/ping")]
fn ping() -> &'static str {
    println!("Received ping");
    "pong!"
}

#[derive(Responder)]
#[response(content_type = "image/webp")]
struct ImageResponse(Vec<u8>);

#[get("/<file..>")]
async fn index(storage: &State<Storage>, file: PathBuf) -> Option<ImageResponse> {
    let time = Instant::now();
    let path = file.as_os_str().to_str()?;
    let res = storage.read(path).await;

    match res {
        Ok(image) => {
            println!("Fetched at {:.2?}", time.elapsed());
            let result: Result<Vec<u8>, libvips::error::Error> =
                services::image::process_image(&image);

            match result {
                Ok(optimised_image) => {
                    println!("Optimized at {:.2?}", time.elapsed());

                    let res = storage
                        .write(
                            path,
                            UploadData {
                                content_type: ContentType::WEBP,
                                body: optimised_image.clone(),
                            },
                        )
                        .await;
                    match res {
                        Ok(_) => println!("Stored optimized image for {} into cache bucket", path),
                        Err(_) => println!(
                            "Could not store optimized image for {} into cache bucket",
                            path
                        ),
                    }

                    Some(ImageResponse(optimised_image))
                }
                Err(error) => {
                    println!("Error during optimization{}", error);
                    Some(ImageResponse(image))
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            None
        }
    }
}

#[get("/generate/<file..>")]
async fn generate(storage: &State<Storage>, file: PathBuf) -> Status {
    let path = file.as_os_str().to_str().unwrap();

    let res = storage.read(path).await;

    match res {
        Ok(image) => {
            let result: Result<Vec<u8>, libvips::error::Error> =
                services::image::process_image(&image);

            match result {
                Ok(optimised_image) => {
                    println!("Optimised");

                    let (file_name_without_ext, _) =
                        file.to_str().unwrap().split_once('.').unwrap();
                    let dest_location = format!("{}.webp", file_name_without_ext);

                    let _res = storage
                        .write(
                            &dest_location,
                            UploadData {
                                content_type: ContentType::WEBP,
                                body: optimised_image.clone(),
                            },
                        )
                        .await;
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

#[get("/fetch/<file..>")]
async fn fetch(storage: &State<Storage>, file: PathBuf) -> Option<ImageResponse> {
    let (file_name_without_ext, _) = file.to_str().unwrap().split_once('.').unwrap();
    let dest_location = format!("{}.webp", file_name_without_ext);

    println!("{:?}", dest_location);
    let cached_image = storage.read_from_cache(&dest_location).await;

    match cached_image {
        Ok(image) => {
            println!("file exists. Returning cached image");
            Some(ImageResponse(image))
        }
        Err(_error) => {
            println!("Error received when fetching the image");
            let path = file.as_os_str().to_str().unwrap();
            let original_image = storage.read(path).await;

            match original_image {
                Ok(original_image) => {
                    let result: Result<Vec<u8>, libvips::error::Error> =
                        services::image::process_image(&original_image);

                    match result {
                        Ok(optimised_image) => {
                            println!("Optimised");

                            let _res = storage
                                .write(
                                    &dest_location,
                                    UploadData {
                                        content_type: ContentType::WEBP,
                                        body: optimised_image.clone(),
                                    },
                                )
                                .await;
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
}

#[launch]
async fn rocket() -> _ {
    // Initialize env variables
    dotenv().ok();

    // Initialize storage service
    let storage: Storage = services::storage::initialize().await.unwrap();

    // Start server
    rocket::build()
        .attach(utils::CORS)
        .attach(AdHoc::on_liftoff("start_consumer", |rocket| {
            Box::pin(async move {
                task::spawn(services::events::start_consumer(rocket.shutdown()));
            })
        }))
        .mount("/", routes![ping])
        .mount("/", routes![index])
        .mount("/", routes![generate])
        .mount("/", routes![fetch])
        .manage(storage)
}
