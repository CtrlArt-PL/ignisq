use crate::storage::{PushMessage, PushQueue};
use chrono::Utc;

impl PushQueue {
    pub async fn enqueue(&self, msg: PushMessage) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO messages (id, token, topic, title, body, send_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&msg.id)
        .bind(&msg.token)
        .bind(&msg.topic)
        .bind(&msg.title)
        .bind(&msg.body)
        .bind(msg.send_at)
        .execute(&self.pool)
        .await?;

        if msg.send_at.is_none() || msg.send_at.unwrap() <= Utc::now() {
            self.notify.notify_one();
        }

        Ok(())
    }

    pub async fn dequeue_batch(&self, max: usize) -> Vec<PushMessage> {
        let res = sqlx::query_as::<_, PushMessage>(
            "SELECT * FROM messages 
            WHERE status = 'pending' 
            AND (send_at IS NULL OR send_at <= datetime('now'))
            LIMIT ?",
        )
        .bind(max as i64)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        if res.is_empty() {
            return vec![];
        }

        let ids: Vec<String> = res.iter().map(|m| m.id.clone()).collect();

        let query = format!(
            "UPDATE messages SET status = 'processing' WHERE id IN ({})",
            ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
        );

        let mut q = sqlx::query(&query);
        for id in ids {
            q = q.bind(id);
        }

        let _ = q.execute(&self.pool).await;

        res
    }

    pub async fn set_status(&self, id: &str, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE messages SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
