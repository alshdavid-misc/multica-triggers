use serde::Deserialize;
use serde::Serialize;

/// A Multica issue.
#[derive(Debug, Deserialize, Serialize)]
pub struct MulticaIssue {
  pub id: String,
  pub identifier: String,
}

/// A comment on a Multica issue.
#[derive(Debug, Deserialize)]
pub struct MulticaComment {
  pub id: String,
}
