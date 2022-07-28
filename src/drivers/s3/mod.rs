use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    error::{GetObjectError, PutObjectError},
    types::ByteStream,
    Client,
};

pub async fn create_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-south-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    Client::new(&config)
}

pub async fn fetch_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
) -> Result<Vec<u8>, GetObjectError> {
    let resp = client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .unwrap();

    let data = resp.body.collect().await.unwrap();
    Ok(data.into_bytes().to_vec())
}

pub async fn upload_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
    data: &'static Vec<u8>,
) -> Result<(), PutObjectError> {
    let body = ByteStream::from_static(data);

    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .send()
        .await
        .unwrap();

    Ok(())
}
