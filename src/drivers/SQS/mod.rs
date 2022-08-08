use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::types::SdkError;
use aws_sdk_sqs::{
    error::{ReceiveMessageError, SendMessageError},
    Client,
};
use std::future::Future;

pub async fn create_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-south-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    Client::new(&config)
}

pub async fn send_message(
    client: &Client,
    queue_url: &str,
    message: &str,
) -> Result<(), SdkError<SendMessageError>> {
    let _ = client
        .send_message()
        .queue_url(queue_url)
        .message_body(message)
        .send()
        .await?;

    Ok(())
}

pub async fn receive_messages<R>(
    client: &Client,
    queue_url: &str,
    handler: fn(String) -> R,
) -> Result<(), SdkError<ReceiveMessageError>>
where
    R: Future<Output = anyhow::Result<()>>,
{
    let response = client
        .receive_message()
        .queue_url(queue_url)
        .max_number_of_messages(10)
        .send()
        .await?;

    for borrowed_message in response.messages.unwrap_or_default() {
        let message = borrowed_message.to_owned();

        if let Some(msg) = borrowed_message.body() {
            log::info!("Received message: {:#?}", msg);
            let res = handler(msg.to_string()).await;
            match res {
                Ok(_) => {
                    if let Some(handle) = message.receipt_handle() {
                        let delete_res = client
                            .delete_message()
                            .queue_url(queue_url)
                            .receipt_handle(handle)
                            .send()
                            .await;

                        if let Err(error) = delete_res {
                            log::error!("{:?}", error);
                        }
                    }

                    return Ok(());
                }
                Err(error) => log::error!("{:?}", error),
            }
        }
    }

    Ok(())
}
