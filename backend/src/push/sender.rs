use std::io::Cursor;
use std::time::Duration;
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
    WebPushError, WebPushMessageBuilder,
};

use crate::entities::subscription::Subscription;
use crate::error::AppError;

#[derive(Debug)]
pub enum DeliveryOutcome {
    Delivered,
    SubscriptionGone,
    Retriable(String),
    Failed(String),
}

pub async fn send_one(
    client: &IsahcWebPushClient,
    subscription: &Subscription,
    vapid_private_pem: &str,
    vapid_subject: &str,
    payload: &str,
    ttl: Option<u32>,
) -> DeliveryOutcome {
    let sub_info = SubscriptionInfo::new(
        subscription.endpoint.clone(),
        subscription.p256dh.clone(),
        subscription.auth.clone(),
    );
    let mut sig_builder =
        match VapidSignatureBuilder::from_pem(Cursor::new(vapid_private_pem.as_bytes()), &sub_info)
        {
            Ok(b) => b,
            Err(e) => return DeliveryOutcome::Failed(format!("vapid builder: {e}")),
        };
    sig_builder.add_claim("sub", vapid_subject.to_string());
    let signature = match sig_builder.build() {
        Ok(s) => s,
        Err(e) => return DeliveryOutcome::Failed(format!("vapid build: {e}")),
    };

    let mut message_builder = WebPushMessageBuilder::new(&sub_info);
    message_builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
    message_builder.set_vapid_signature(signature);
    if let Some(secs) = ttl {
        message_builder.set_ttl(secs);
    }

    let message = match message_builder.build() {
        Ok(m) => m,
        Err(e) => return DeliveryOutcome::Failed(format!("build message: {e}")),
    };

    match client.send(message).await {
        Ok(_) => DeliveryOutcome::Delivered,
        Err(e) => map_error(e),
    }
}

fn map_error(err: WebPushError) -> DeliveryOutcome {
    let msg = err.to_string();
    if msg.contains("404") || msg.contains("410") || msg.contains("Gone") {
        return DeliveryOutcome::SubscriptionGone;
    }
    if msg.contains("429")
        || msg.contains(" 5")
        || msg.contains("Too Many")
        || msg.contains("Service Unavailable")
    {
        return DeliveryOutcome::Retriable(msg);
    }

    DeliveryOutcome::Failed(msg)
}

pub async fn send_with_retry(
    client: &IsahcWebPushClient,
    subscription: &Subscription,
    vapid_private_pem: &str,
    vapid_subject: &str,
    payload: &str,
    ttl: Option<u32>,
) -> Result<DeliveryOutcome, AppError> {
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        match send_one(
            client,
            subscription,
            vapid_private_pem,
            vapid_subject,
            payload,
            ttl,
        )
        .await
        {
            DeliveryOutcome::Delivered => return Ok(DeliveryOutcome::Delivered),
            DeliveryOutcome::SubscriptionGone => return Ok(DeliveryOutcome::SubscriptionGone),
            DeliveryOutcome::Failed(reason) => {
                if attempt >= 2 {
                    return Ok(DeliveryOutcome::Failed(reason));
                }
            }
            DeliveryOutcome::Retriable(reason) => {
                if attempt >= 3 {
                    return Ok(DeliveryOutcome::Retriable(reason));
                }
                let backoff = Duration::from_millis(500 * (1 << (attempt - 1)));
                tracing::warn!(attempt, backoff_ms = backoff.as_millis(), %reason, "push retriable error, retrying");
                tokio::time::sleep(backoff).await;
            }
        }
    }
}
