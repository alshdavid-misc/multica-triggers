use std::fmt::Write;

use sha2::Digest;
use sha2::Sha256;

/// Lowercase hex SHA-256 digest of `input` (64 characters).
pub fn hash(input: &str) -> String {
  let digest = Sha256::digest(input.as_bytes());
  let mut out = String::with_capacity(digest.len() * 2);

  for byte in digest.iter() {
    let _ = write!(out, "{byte:02x}");
  }

  out
}

const KEY_LEN: usize = 64;

/// Digest trimmed and prefixed so the first character is a letter — the shape
/// Multica metadata keys require.
pub fn hash_key(input: &str) -> String {
  let digest = hash(input);
  format!("K{}", &digest[..KEY_LEN - 1])
}
