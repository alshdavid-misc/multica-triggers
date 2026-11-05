use hmac::Hmac;
use hmac::Mac;
use sha2::Sha256;

/// Checks a GitHub `X-Hub-Signature-256` header (`sha256=<hex>`) against `body`.
pub fn verify_github_signature(
  secret: &str,
  body: &[u8],
  signature: &str,
) -> bool {
  let Some(signature) = signature
    .strip_prefix("sha256=")
    .and_then(|hex| hex::decode(hex).ok())
  else {
    return false;
  };

  let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
    return false;
  };

  mac.update(body);
  mac.verify_slice(&signature).is_ok()
}
