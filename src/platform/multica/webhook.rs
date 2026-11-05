use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;

pub async fn send_webhook<T: Serialize>(
  webhook: &str,
  payload: &T,
) -> anyhow::Result<()> {
  let body = serde_json::to_string_pretty(payload)?;

  let response = Client::new()
    .post(webhook)
    .header(CONTENT_TYPE, "application/json")
    .body(body)
    .send()
    .await
    .inspect_err(|error| eprintln!("webhook request failed {webhook}: {error:?}"))?;

  let status = response.status();
  if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    eprintln!("webhook error {status} {webhook}: {text}");
    anyhow::bail!("webhook error {status}: {text}");
  }

  Ok(())
}
