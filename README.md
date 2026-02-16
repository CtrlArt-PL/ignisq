# ⚡ IgnisQ: The High-Velocity FCM Relay

**IgnisQ** is an industrial-grade, asynchronous Firebase Cloud Messaging (FCM) relay engine engineered in **Rust**.

It serves as a high-performance buffer and scheduling layer between your backend services and Google's FCM infrastructure. By offloading the complexity of rate limiting, batching, and persistent queuing to a dedicated, low-footprint service, IgnisQ ensures that your notifications are delivered with millisecond precision and zero message loss.

**Why IgnisQ?**
* **Decouple your Backend:** Send-and-forget logic—IgnisQ handles the rest.
* **Guaranteed Delivery:** SQLite-backed persistence ensures no notification is lost, even through crashes or restarts.
* **Optimized Throughput:** Intelligent batching and non-blocking I/O maximize your FCM quota utilization without hitting rate limits.


## 🚀 Features
- **Extreme Performance:** Built with Rust & Tokio, capable of processing thousands of messages with sub-millisecond overhead.
- **Smart Queuing:** Uses SQLite to ensure no notifications are lost during restarts.
- **Batch Processing:** High-speed delivery handling up to 500 messages in a single API call.
- **Efficient Relay Loop:** Intelligent non-blocking worker that wakes up instantly on new tasks or polls periodically.
- **Scheduling:** Future-dated delivery support via `send_at` timestamps.
- **Rate Limiting:** Intelligent throttling (min 20ms per batch) to respect FCM quotas and prevent IP blacklisting.
- **Automatic Cleanup:** Automated background service purges `sent` and `failed` records older than 24h to keep the database slim.
- **Security:** Robust Bearer Token authentication for all push endpoints.
- **Color-coded Logging:** Real-time observability with timestamps and module-specific colors for easy debugging.


## 🛠 Architecture
The service consists of three main components:
1. **Axum API:** REST interface for message ingestion.
2. **FCM Worker:** Queue processor that communicates with Google V1 API.
3. **Database Cleaner:** Maintenance task for database hygiene.


## 🚥 API Documentation
All endpoints (except `/health`) require authentication via Bearer Token.
**Header:** `Authorization: Bearer <YOUR_API_TOKEN>`

### 1. Health Check
Simple endpoint to verify if the service is up and running.
* **URL:** `/health`
* **Method:** `GET`
* **Response:** `200 OK` | `All ok`

### 2. Single Push Notification
Queue a single message for a specific device or topic.
* **URL:** `/push`
* **Method:** `POST`

**Send to Device Token**
```json
{
  "token": "fCM_dEvIcE_tOkEn_123",
  "title": "Welcome!",
  "body": "Thanks for joining IgnisQ."
}
```

**Send to Topic**
```json
{
  "topic": "fCM_dEvIcE_tOkEn_123",
  "title": "Big Sale!",
  "body": "50% off for all items!"
}
```

**Scheduled Push**
```json
{
  "token": "fCM_dEvIcE_tOkEn_123",
  "title": "Delayed Hello",
  "body": "This notification was scheduled for later.",
  "send_at": "2026-02-25T10:00:00Z"
}
```

### 3. Bulk Push (Many)
Queue multiple messages in a single request. The payload is an **array of objects** with the same structure as in the Single Push endpoint.
* **URL:** `/push/many`
* **Method:** `POST`
* **Constraint:** `Maximum 500 messages per request.`

**Example Payload:**
```json
[
  {
    "token": "fCM_dEvIcE_tOkEn_123",
    "title": "First Message",
    "body": "Hello Alpha"
  },
  {
    "topic": "news",
    "title": "Second Message",
    "body": "Hello World!",
    "send_at": "2026-02-15T20:00:00Z"
  }
]
```

## 📦 Docker Deployment
The official IgnisQ image is available on Docker Hub. It is highly optimized, with a total size of ~24MB and a runtime footprint of ~20MB RAM.

To get the latest stable version of IgnisQ:
```bash
docker pull ctrlart/ignisq:latest
```


## ⚙️ Configuration (.env)
The service requires the following environment variables. Create a `.env` file in the root directory:

```env
# Mode: set to 'true' to enable FCM "validate_only" mode (test dry-run)
# If false or unset, notifications will be delivered to real devices.
DEBUG=false

# Security token for your API (used in Bearer Auth)
API_TOKEN=your_secure_token_here

# Server host (defaults to 0.0.0.0)
HOST=0.0.0.0

# Google Service Account JSON
# IMPORTANT: This must be the entire JSON string. 
# Ensure the "private_key" field contains literal '\n' characters for correct parsing.
FCM_SERVICE_ACCOUNT_JSON={"type": "service_account", "project_id": "...", "private_key": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n", ...}
```


## ⚖️ License
This project is licensed under the **Business Source License 1.1 (BSL 1.1)**.
* **Personal & Internal Use:** Always free.
* **Commercial Use:** Free for internal business purposes.
* **SaaS Restriction:** You cannot sell IgnisQ as a managed service.
* **Future Open Source:** This version will convert to Apache 2.0 on 2029-01-01.