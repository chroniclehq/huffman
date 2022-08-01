use crate::drivers::S3;
pub use crate::drivers::S3::UploadData;
use anyhow::{anyhow, Result};
use aws_sdk_s3::Client;
use std::env;

pub struct Storage {
    _client: Client,
    _source: String,
    _dest: String,
}

impl Storage {
    pub async fn read(&self, key: &str) -> Result<Vec<u8>> {
        let result = S3::fetch_object(&self._client, &self._source, &key).await;

        match result {
            Ok(data) => Ok(data),
            Err(error) => {
                println!("{:?}", error);
                Err(anyhow!("Could not read object"))
            }
        }
    }

    pub async fn write(&self, key: &str, value: UploadData) -> Result<()> {
        let result = S3::upload_object(&self._client, &self._dest, &key, value).await;
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                println!("{:?}", error);
                Err(anyhow!("Could not write object"))
            }
        }
    }

    pub async fn read_from_cache(&self, key: &str) -> Result<Vec<u8>> {
        let result = S3::fetch_object(&self._client, &self._dest, &key).await;

        match result {
            Ok(data) => Ok(data),
            Err(error) => {
                println!("{:?}", error);
                Err(anyhow!("Could not read object"))
            }
        }
    }
}

pub async fn initialize() -> Result<Storage> {
    let client = S3::create_client().await;
    let source = env::var("SOURCE_BUCKET")?;
    let dest = env::var("CACHE_BUCKET")?;

    Ok(Storage {
        _client: client,
        _source: source,
        _dest: dest,
    })
}
