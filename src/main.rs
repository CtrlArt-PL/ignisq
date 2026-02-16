mod api;
mod config;
mod fcm;
mod storage;
mod utils;

use colored::*;

fn print_banner() {
    let banner = r#"
    ▗▄▄▄▖ ▗▄▄▖▗▖  ▗▖▗▄▄▄▖ ▗▄▄▖▗▄▄▄▖ 
      █  ▐▌   ▐▛▚▖▐▌  █  ▐▌   ▐▌ ▐▌ 
      █  ▐▌▝▜▌▐▌ ▝▜▌  █   ▝▀▚▖▐▌ ▐▌ 
    ▗▄█▄▖▝▚▄▞▘▐▌  ▐▌▗▄█▄▖▗▄▄▞▘▐▙▄▟▙▖
    "#;

    println!("{}", banner.red());
    println!(
        "       {}",
        "IGNISQ Push Service v0.1.0".green().bold()
    );
    println!(" ");
}

#[tokio::main]
async fn main() {
    print_banner();

    let cfg = config::load();
    let queue = storage::PushQueue::new().await;

    // Cleaner
    let cleaner_queue = queue.clone();
    tokio::spawn(async move {
        cleaner_queue.start_cleaner().await;
    });

    // FCM
    let worker_queue = queue.clone();
    tokio::spawn(fcm::start(worker_queue, cfg.debug, cfg.service_account_key));

    // API
    api::start(queue, cfg.api_token, cfg.host).await;
}
