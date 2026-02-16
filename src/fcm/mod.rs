mod client;

use crate::fcm::client::FcmStatus;
use crate::storage::PushQueue;
use crate::utils::get_ts;
use colored::*;
use tokio::time::{Duration, sleep};

pub async fn start(queue: PushQueue, debug: bool, service_acc_key: yup_oauth2::ServiceAccountKey) {
    let min_batch_duration = Duration::from_millis(20);

    let client = client::FcmClient::new(debug, &service_acc_key)
        .await
        .expect(&format!(
            "{} {} Failed to initialize client",
            get_ts(),
            "[FCM]".magenta().bold()
        ));

    let notify = queue.get_notify();

    println!(
        "{} {} Worker is alive and listening for tasks",
        get_ts(),
        "[FCM]".magenta().bold()
    );

    loop {
        let start_time = tokio::time::Instant::now();

        let messages = queue.dequeue_batch(50).await;

        if messages.is_empty() {
            tokio::select! {
                _ = notify.notified() => {
                }
                _ = sleep(Duration::from_secs(30)) => {
                }
            }
            continue;
        }

        println!(
            "{} {} Processing {} messages...",
            get_ts(),
            "[FCM]".magenta().bold(),
            messages.len().to_string()
        );

        let results = client.send_bulk(messages).await;

        for (id, res) in results {
            match res {
                Ok(FcmStatus::Success) => {
                    let _ = queue.set_status(&id, "sent").await;
                }
                Ok(FcmStatus::Error(e)) => {
                    eprintln!(
                        "{} {} {} for {}: {}",
                        get_ts(),
                        "[FCM]".magenta().bold(),
                        "Critical error",
                        id,
                        e.to_string()
                    );
                    let _ = queue.set_status(&id, "failed").await;
                }
                Ok(FcmStatus::Retry) | Err(_) => {
                    let _ = queue.set_status(&id, "pending").await;
                }
            }
        }

        let elapsed = start_time.elapsed();
        if elapsed < min_batch_duration {
            sleep(min_batch_duration - elapsed).await;
        }
    }
}
