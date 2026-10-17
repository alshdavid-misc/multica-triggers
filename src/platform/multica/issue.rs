use serde::Deserialize;

/// A Multica issue.
#[derive(Debug, Deserialize)]
pub struct MulticaIssue {
  pub id: String,
  pub identifier: String,
}
