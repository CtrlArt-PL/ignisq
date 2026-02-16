use axum::{
    Json, Router,
    http::{Request, StatusCode, header},
    middleware::{self, Next},
    routing::{get, post},
};
use axum_server::bind;
use chrono::{DateTime, Utc};
use colored::*;
use serde::Deserialize;
use std::net::IpAddr;
use std::{net::SocketAddr, sync::Arc};
use uuid::Uuid;

use crate::storage::{PushMessage, PushQueue};
use crate::utils::get_ts;

#[derive(Deserialize)]
struct PushRequest {
    token: Option<String>,
    topic: Option<String>,
    title: String,
    body: String,
    send_at: Option<DateTime<Utc>>,
}

pub async fn start(queue: PushQueue, api_token: String, host: IpAddr) {
    let shared_token = Arc::new(api_token);

    let protected_routes = Router::new()
        .route(
            "/push",
            post({
                let q = queue.clone();
                move |Json(payload): Json<PushRequest>| {
                    let q_inner = q.clone();
                    async move { handle_push(payload, q_inner).await }
                }
            }),
        )
        .route(
            "/push/many",
            post({
                let q = queue.clone();
                move |Json(payload): Json<Vec<PushRequest>>| {
                    let q_inner = q.clone();
                    async move {
                        if payload.len() > 500 {
                            return (
                                StatusCode::BAD_REQUEST,
                                "Batch too big! Exceeded limit of 500.".to_string(),
                            );
                        }
                        handle_push_many(payload, q_inner).await
                    }
                }
            }),
        )
        .layer(middleware::from_fn(move |req: Request<_>, next: Next| {
            let auth_token = shared_token.clone();
            async move {
                let auth_header = req
                    .headers()
                    .get(header::AUTHORIZATION)
                    .and_then(|h| h.to_str().ok());

                if let Some(auth_str) = auth_header {
                    if auth_str == format!("Bearer {}", auth_token) {
                        return Ok(next.run(req).await);
                    }
                }
                Err(StatusCode::UNAUTHORIZED)
            }
        }));

    let app = Router::new()
        .route("/health", get(|| async { (StatusCode::OK, "All ok") }))
        .merge(protected_routes);

    println!(
        "{} {} Running on {}",
        get_ts(),
        "[API]".cyan().bold(),
        format!("http://{}:9191", host).underline().bright_white()
    );

    let addr = SocketAddr::new(host, 9191);
    bind(addr).serve(app.into_make_service()).await.unwrap();
}

async fn handle_push(payload: PushRequest, queue: PushQueue) -> (StatusCode, String) {
    if payload.token.is_none() && payload.topic.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            "Missing both 'token' and 'topic'. One is required.".into(),
        );
    }

    let msg = PushMessage {
        id: Uuid::new_v4().to_string(),
        token: payload.token,
        topic: payload.topic,
        title: payload.title,
        body: payload.body,
        send_at: payload.send_at,
    };

    let _ = queue.enqueue(msg).await;

    (StatusCode::OK, "Task queued".into())
}

async fn handle_push_many(payload: Vec<PushRequest>, queue: PushQueue) -> (StatusCode, String) {
    let count = payload.len();

    for item in payload {
        if item.token.is_none() && item.topic.is_none() {
            return (
                StatusCode::BAD_REQUEST,
                "One of the messages is missing both 'token' and 'topic'.".into(),
            );
        }

        let msg = PushMessage {
            id: Uuid::new_v4().to_string(),
            token: item.token,
            topic: item.topic,
            title: item.title,
            body: item.body,
            send_at: item.send_at,
        };

        let _ = queue.enqueue(msg).await;
    }

    (StatusCode::OK, format!("Queued {} tasks", count))
}
