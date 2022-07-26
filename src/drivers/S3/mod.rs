use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    error::{GetObjectError, PutObjectError},
    types::{ByteStream,SdkError},
    Client,
};
use rocket::http::ContentType;

pub async fn create_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-south-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    Client::new(&config)
}

pub async fn fetch_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
) -> Result<Vec<u8>, SdkError<GetObjectError>> {
    let resp = client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await?;

    let data = resp.body.collect().await.unwrap();
    Ok(data.into_bytes().to_vec())
}

pub struct UploadData {
    pub body: Vec<u8>,
    pub content_type: ContentType,
}

pub async fn upload_object<'a>(
    client: &Client,
    bucket_name: &str,
    key: &str,
    data: UploadData,
) -> Result<(), PutObjectError> {
    let stream = ByteStream::from(data.body.clone());

    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(stream)
        .content_type(data.content_type.to_string())
        .send()
        .await
        .unwrap();

    Ok(())
}
