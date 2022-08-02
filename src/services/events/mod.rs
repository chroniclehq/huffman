use rocket::tokio::select;
use rocket::{tokio::time, Shutdown};

pub async fn start_consumer(mut shutdown: Shutdown) {
    loop {
        select! {
            _ = time::sleep(time::Duration::from_secs(5)) =>
                println!("Polling from consumer"),
            _ = &mut shutdown => {
                println!("Shutting down consumer");
                break;
            },
        }
    }
}
