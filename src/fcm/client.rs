use anyhow::Result;
use futures::StreamExt;
use hyper_util::client::legacy::connect::HttpConnector;
use reqwest::Client;
use yup_oauth2::ServiceAccountKey;
use yup_oauth2::authenticator::{Authenticator, ServiceAccountAuthenticator};
use yup_oauth2::hyper_rustls::HttpsConnector;

use crate::storage::PushMessage;

pub struct FcmClient {
    debug: bool,
    project_id: String,
    auth: Authenticator<HttpsConnector<HttpConnector>>,
    http: Client,
}

pub enum FcmStatus {
    Success,
    Error(String),
    Retry,
}

impl FcmClient {
    pub async fn new(debug: bool, service_account_key: &ServiceAccountKey) -> Result<FcmClient> {
        let key_clone = service_account_key.clone();

        let project_id = key_clone
            .project_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("project_id missing in service account key"))?;

        let auth = ServiceAccountAuthenticator::builder(key_clone)
            .build()
            .await?;

        Ok(FcmClient {
            debug: debug,
            project_id: project_id,
            auth,
            http: Client::new(),
        })
    }

    pub async fn send_single(&self, message: PushMessage) -> Result<FcmStatus> {
        let token = self
            .auth
            .token(&["https://www.googleapis.com/auth/firebase.messaging"])
            .await?;

        let bearer = token
            .token()
            .ok_or_else(|| anyhow::anyhow!("Received an empty access token from Google"))?
            .to_string();

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        let mut message_obj = serde_json::json!({
            "notification": {
                "title": message.title,
                "body": message.body
            }
        });

        if let Some(topic) = message.topic {
            message_obj["topic"] = serde_json::json!(topic);
        } else if let Some(token) = message.token {
            message_obj["token"] = serde_json::json!(token);
        }

        let payload = serde_json::json!({
            "validate_only": self.debug,
            "message": message_obj
        });

        let resp = self
            .http
            .post(url)
            .bearer_auth(&bearer)
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status();

        if status_code.is_success() {
            return Ok(FcmStatus::Success);
        }

        let error_text = resp.text().await?;

        match status_code.as_u16() {
            400 | 404 => {
                // INVALID_ARGUMENT lub NOT_FOUND (bad token/topic)
                Ok(FcmStatus::Error(format!("Critical: {}", error_text)))
            }
            429 | 500 | 503 => {
                // internal server error or to many requests
                Ok(FcmStatus::Retry)
            }
            _ => Ok(FcmStatus::Error(error_text)),
        }
    }

    pub async fn send_bulk(&self, messages: Vec<PushMessage>) -> Vec<(String, Result<FcmStatus>)> {
        futures::stream::iter(messages)
            .map(|msg| async move {
                let id = msg.id.clone();
                let res = self.send_single(msg).await;
                (id, res)
            })
            .buffer_unordered(50)
            .collect::<Vec<_>>()
            .await
    }
}
