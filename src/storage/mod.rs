mod db;

use crate::utils::get_ts;
use chrono::{DateTime, Utc};
use colored::*;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushMessage {
    pub id: String,
    pub token: Option<String>,
    pub topic: Option<String>,
    pub title: String,
    pub body: String,
    pub send_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct PushQueue {
    pool: SqlitePool,
    notify: Arc<Notify>,
}

impl PushQueue {
    pub async fn new() -> Self {
        let database_url = "sqlite:data/ignisq.db?mode=rwc";

        let options = SqliteConnectOptions::from_str(database_url)
            .expect(&format!(
                "{} {} {}",
                get_ts(),
                "[STORAGE]".green().bold(),
                "Invalid database URL"
            ))
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await.expect(&format!(
            "{} {} {}",
            get_ts(),
            "[STORAGE]".green().bold(),
            "Failed to connect to SQLite"
        ));

        Self::setup_database(&pool).await;

        println!(
            "{} {} Database initialized (SQLite)",
            get_ts(),
            "[STORAGE]".green().bold()
        );

        Self {
            pool,
            notify: Arc::new(Notify::new()),
        }
    }

    async fn setup_database(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                token TEXT,
                topic TEXT,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                send_at DATETIME,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
        )
        .execute(pool)
        .await
        .expect(&format!(
            "{} {} Failed to setup database",
            get_ts(),
            "[STORAGE]".green().bold()
        ));
    }

    pub fn get_notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    pub async fn start_cleaner(&self) {
        let interval_secs = 3600;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        println!(
            "{} {} Cleaner service started. Interval: {}s",
            get_ts(),
            "[CLEANUP]".yellow().bold(),
            interval_secs.to_string().bright_white()
        );

        loop {
            interval.tick().await;

            match sqlx::query(
                "DELETE FROM messages 
                    WHERE status IN ('sent', 'failed') 
                    AND created_at < datetime('now', '-1 day')",
            )
            .execute(&self.pool)
            .await
            {
                Ok(res) => {
                    let rows = res.rows_affected();
                    if rows > 0 {
                        println!(
                            "{} {} Removed {} stale records (sent/failed)",
                            get_ts(),
                            "[CLEANUP]".yellow().bold(),
                            rows
                        );
                    } else {
                        println!(
                            "{} {} Cycle finished: no stale records to remove",
                            get_ts(),
                            "[CLEANUP]".yellow().bold()
                        );
                    }
                }
                Err(e) => eprintln!(
                    "{} {} {} Database error: {}",
                    get_ts(),
                    "[CLEANUP]".yellow().bold(),
                    "Cleanup failed!",
                    e.to_string()
                ),
            }
        }
    }
}
