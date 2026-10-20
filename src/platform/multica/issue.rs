use serde::Deserialize;

/// A Multica issue.
#[derive(Debug, Deserialize)]
pub struct MulticaIssue {
  pub id: String,
  pub identifier: String,
}

/// A comment on a Multica issue.
#[derive(Debug, Deserialize)]
pub struct MulticaComment {
  pub id: String,
  pub content: String,
}
