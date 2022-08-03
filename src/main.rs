#[macro_use]
extern crate rocket;
extern crate dotenv;

mod drivers;
mod services;
mod utils;

use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::tokio::task;
use rocket::State;
use services::events::{consume_events, EventChannel};
use services::storage::Storage;
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
async fn fetch(
    storage: &State<Storage>,
    channel: &State<EventChannel>,
    file: PathBuf,
) -> Option<ImageResponse> {
    let path = file.as_os_str().to_str();
    let time = Instant::now();

    match path {
        Some(key) => {
            let file_name_without_ext = utils::get_path_without_ext(key);
            let target_path = format!("default/{}.webp", file_name_without_ext);
            let cached_image = storage.read_from_cache(&target_path).await;

            match cached_image {
                Ok(image) => {
                    println!(
                        "Variant found for {} at {:2?}. Returning from cache",
                        key,
                        time.elapsed()
                    );
                    Some(ImageResponse(image))
                }
                Err(_error) => {
                    let original_image = storage.read(key).await;

                    match original_image {
                        Ok(original_image) => {
                            let result: Result<Vec<u8>, libvips::error::Error> =
                                services::image::optimize(&original_image);

                            match result {
                                Ok(optimised_image) => {
                                    println!("Optimised {} at {:2?}", key, time.elapsed());

                                    let _ = channel.send(key).await;

                                    Some(ImageResponse(optimised_image))
                                }
                                Err(error) => {
                                    println!("Error during optimization {}", error);
                                    Some(ImageResponse(original_image))
                                }
                            }
                        }
                        Err(error) => {
                            println!("Could not find image {}", error);
                            None
                        }
                    }
                }
            }
        }
        None => {
            println!("Missing path");
            None
        }
    }
}

#[get("/generate/<file..>")]
async fn generate(storage: &State<Storage>, file: PathBuf) -> Status {
    let path = file.as_os_str().to_str();
    match path {
        // TODO @harris: Figure out a way to move this onto a blocking thread
        Some(key) => match services::image::generate(key, &storage).await {
            Ok(_) => Status::Ok,
            Err(error) => {
                println!("{}", error);
                Status::InternalServerError
            }
        },
        None => Status::InternalServerError,
    }
}

#[launch]
async fn rocket() -> _ {
    // Initialize env variables
    dotenv().ok();

    // Initialize services
    let storage: Storage = services::storage::initialize().await.unwrap();
    let channel: EventChannel = services::events::create_channel().await.unwrap();

    // Start server
    rocket::build()
        .manage(storage)
        .manage(channel)
        .attach(utils::CORS)
        .attach(AdHoc::on_liftoff("start_consumer", |rocket| {
            Box::pin(async {
                task::spawn(consume_events(rocket.shutdown()));
            })
        }))
        .mount("/", routes![ping])
        .mount("/", routes![fetch])
        .mount("/", routes![generate])
}
