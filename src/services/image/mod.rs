use crate::utils;

use super::storage::{Storage, UploadData};
use anyhow;
use enum_map::{enum_map, Enum, EnumMap};
use libvips::ops::webpsave_buffer_with_opts;
use libvips::VipsImage;
use libvips::{self, ops};
use rocket::http::ContentType;

#[derive(Enum)]
pub enum Variants {
    Default,
}

pub fn get_variant_path(variant: Variants) -> String {
    let folder_name = "image_optimizer";
    let paths: EnumMap<Variants, &str> = enum_map! {
        Variants::Default => "default"
    };
    format!("{}/{}", folder_name, paths[variant].to_string())
}

pub fn optimize(buffer: &Vec<u8>) -> libvips::Result<Vec<u8>> {
    let source = VipsImage::new_from_buffer(buffer, "").expect("Error during VipsImage init");
    let options = ops::WebpsaveBufferOptions {
        q: 50,
        strip: true,
        reduction_effort: 2,
        ..ops::WebpsaveBufferOptions::default()
    };
    webpsave_buffer_with_opts(&source, &options)
}

pub async fn generate(key: &str, storage: &Storage) -> anyhow::Result<()> {
    let file_name_without_ext = utils::get_path_without_ext(key);
    let variant_path = get_variant_path(Variants::Default);

    let target_path = format!("{}/{}.webp", variant_path, file_name_without_ext);
    let cached_image = storage.read_from_cache(&target_path).await;

    match cached_image {
        Ok(_) => {
            println!("Variant found for {}. Skipping generate flow.", key);
            Ok(())
        }
        Err(_error) => {
            let image = storage.read(key).await?;
            let result: Result<Vec<u8>, libvips::error::Error> = optimize(&image);

            match result {
                Ok(optimised_image) => {
                    let file_name_without_ext = utils::get_path_without_ext(key);
                    let variant_path = get_variant_path(Variants::Default);
                    let target_key = format!("{}/{}.webp", variant_path, file_name_without_ext);

                    storage
                        .write(
                            &target_key,
                            UploadData {
                                content_type: ContentType::WEBP,
                                body: optimised_image,
                            },
                        )
                        .await?;

                    Ok(())
                }
                Err(error) => Err(anyhow::anyhow!(format!(
                    "Error during optimization. Key: {}, Error: {:?}",
                    key, error
                ))),
            }
        }
    }
}
