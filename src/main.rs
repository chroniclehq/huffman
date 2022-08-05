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
use services::events::{message::Message, EventChannel};
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
            let variant_path =
                services::image::get_variant_path(services::image::Variants::Default);

            let target_path = format!("{}/{}.webp", variant_path, file_name_without_ext);
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

                                    if let Ok(_) = channel
                                        .send_message(&Message {
                                            url: key.to_string(),
                                        })
                                        .await
                                    {
                                        println!(
                                            "Queued {} for caching at {:2?}",
                                            key,
                                            time.elapsed()
                                        );
                                    }

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
            println!("Missing path in fetch request");
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
                    println!("{}", error);
                    Status::InternalServerError
                }
            }
        }
        None => {
            println!("Missing path in generate request");
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

    // Start server
    rocket::build()
        .manage(storage)
        .manage(channel)
        .attach(utils::CORS)
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
