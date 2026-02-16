use colored::*;
use dotenv::dotenv;
use std::net::IpAddr;
use std::{env, str::FromStr};
use yup_oauth2::ServiceAccountKey;

use crate::utils::get_ts;

pub struct Config {
    pub debug: bool,
    pub service_account_key: ServiceAccountKey,
    pub api_token: String,
    pub host: IpAddr,
}

pub fn load() -> Config {
    dotenv().ok();

    let debug = env::var("DEBUG")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if debug {
        println!(
            "{} {} Debug mode enabled",
            get_ts(),
            "[CONFIG]".white().bold()
        )
    }

    let json_str = env::var("FCM_SERVICE_ACCOUNT_JSON").expect(&format!(
        "{} {} {}",
        get_ts(),
        "[CONFIG]".white().bold(),
        "FCM_SERVICE_ACCOUNT_JSON must be set in environment"
    ));

    let service_account_key: ServiceAccountKey = serde_json::from_str(&json_str).expect(&format!(
        "{} {} {}",
        get_ts(),
        "[CONFIG]".white().bold(),
        "Invalid JSON in FCM_SERVICE_ACCOUNT_JSON"
    ));

    let api_token = env::var("API_TOKEN").expect(&format!(
        "{} {} {}",
        get_ts(),
        "[CONFIG]".white().bold(),
        "API_TOKEN must be set for security"
    ));

    let host_str = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let host = IpAddr::from_str(&host_str).expect(&format!(
        "{} {} {}",
        get_ts(),
        "[CONFIG]".white().bold(),
        "HOST must be a valid IP address (e.g., 127.0.0.1 or 0.0.0.0)"
    ));

    println!(
        "{} {} Configuration loaded successfully",
        get_ts(),
        "[CONFIG]".white().bold()
    );

    Config {
        debug,
        service_account_key,
        api_token,
        host,
    }
}
