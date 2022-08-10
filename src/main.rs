#[macro_use]
extern crate rocket;
extern crate dotenv;

mod drivers;
mod services;
mod utils;

use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::http::{ContentType, Status};
use rocket::tokio::task;
use rocket::State;
use services::events::{message::Message, EventChannel};
use services::storage::Storage;
use std::path::PathBuf;
use std::time::Instant;
use utils::http::{CacheControl, ImageResponse, TextResponse, CORS};

#[get("/ping")]
fn ping() -> TextResponse {
    log::info!("Received ping");
    TextResponse::new("pong")
}

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
            let ext = utils::get_ext_from_path(key).unwrap_or_else(|| "png");
            let is_allowed = utils::is_allowed_type(ext);

            match is_allowed {
                Ok(_) => {
                    let variant_path =
                        services::image::get_variant_path(services::image::Variants::Default);
                    let target_path = format!("{}/{}.webp", variant_path, file_name_without_ext);
                    let cached_image = storage.read_from_cache(&target_path).await;
                    match cached_image {
                        Ok(image) => {
                            log::info!(
                                "Variant found for {} at {:2?}. Returning from cache",
                                key,
                                time.elapsed()
                            );
                            Some(ImageResponse::new(
                                image,
                                ContentType::WEBP,
                                CacheControl::Default,
                            ))
                        }
                        Err(_error) => {
                            let original_image = storage.read(key).await;

                            match original_image {
                                Ok(original_image) => {
                                    let result: Result<Vec<u8>, libvips::error::Error> =
                                        services::image::optimize(&original_image);

                                    match result {
                                        Ok(optimised_image) => {
                                            log::info!(
                                                "Optimised {} at {:2?}",
                                                key,
                                                time.elapsed()
                                            );

                                            if let Ok(_) = channel
                                                .send_message(&Message {
                                                    url: key.to_string(),
                                                })
                                                .await
                                            {
                                                log::info!(
                                                    "Queued {} for caching at {:2?}",
                                                    key,
                                                    time.elapsed()
                                                );
                                            }

                                            Some(ImageResponse::new(
                                                optimised_image,
                                                ContentType::WEBP,
                                                CacheControl::Default,
                                            ))
                                        }
                                        Err(error) => {
                                            log::error!("Error during optimization {}", error);

                                            Some(ImageResponse::new(
                                                original_image,
                                                ContentType::from_extension(ext)
                                                    .unwrap_or_default(),
                                                CacheControl::NoCache,
                                            ))
                                        }
                                    }
                                }
                                Err(error) => {
                                    log::warn!("Could not find image {}", error);
                                    None
                                }
                            }
                        }
                    }
                }
                Err(error) => {
                    log::warn!("{}", error);
                    let original_image = storage.read(key).await;
                    match original_image {
                        Ok(original_image) => Some(ImageResponse::new(
                            original_image,
                            ContentType::from_extension(ext).unwrap_or_default(),
                            CacheControl::Default,
                        )),
                        Err(error) => {
                            log::warn!("Could not find image {}", error);
                            None
                        }
                    }
                }
            }
        }
        None => {
            log::warn!("Missing path in fetch request");
            None
        }
    }
}

#[get("/generate/<file..>")]
async fn generate(channel: &State<EventChannel>, file: PathBuf) -> Status {
    let path = file.as_os_str().to_str();
    match path {
        Some(key) => {
            match channel
                .send_message(&Message {
                    url: key.to_string(),
                })
                .await
            {
                Ok(_) => Status::Ok,
                Err(error) => {
                    log::error!("{}", error);
                    Status::InternalServerError
                }
            }
        }
        None => {
            log::warn!("Missing path in generate request");
            Status::InternalServerError
        }
    }
}

#[launch]
async fn rocket() -> _ {
    // Load env variables
    dotenv().ok();

    // Initialize services
    let storage: Storage = services::storage::initialize().await.unwrap();
    let channel: EventChannel = services::events::initialize().await.unwrap();

    let _logger = services::logger::initialize().await;

    // Start server
    rocket::build()
        .manage(storage)
        .manage(channel)
        .attach(CORS)
        .attach(AdHoc::on_liftoff("start_consumer", |rocket| {
            // Box::pin is required when spawning threads inside a fairing:
            // https://github.com/SergioBenitez/Rocket/issues/1640
            // https://github.com/SergioBenitez/Rocket/issues/1303
            Box::pin(async {
                let shutdown = rocket.shutdown();
                task::spawn(async {
                    let channel: EventChannel = services::events::initialize().await.unwrap();
                    let _ = channel.listen(shutdown).await;
                    ()
                });
            })
        }))
        .mount("/", routes![ping])
        .mount("/", routes![fetch])
        .mount("/", routes![generate])
}
