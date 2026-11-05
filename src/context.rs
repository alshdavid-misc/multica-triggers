use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use octocrab::Octocrab;

use crate::platform::github::CommentKind;

#[derive(Clone)]
pub struct AppContext {
  pub multica_url: String,
  pub multica_token: String,
  pub multica_workspace_id: String,
  pub multica_project_id: Option<String>,
  pub github_token: String,
  pub github_webhook_secret: String,
  pub github_user_reply_list: Option<Vec<u64>>,
  pub github_machine_user_id: u64,
  pub multica_pull_request_comment_webhook: Option<String>,
  pub multica_pull_request_sync_webhook: Option<String>,
  pub github: Arc<Octocrab>,
  /// Autopilot runs, keyed by run id, whose comment still carries a 👀 reaction.
  pub pending_runs: Arc<Mutex<HashMap<String, PendingRun>>>,
}

#[derive(Debug, Clone)]
pub struct PendingRun {
  pub autopilot_id: String,
  pub repo: String,
  pub comment_kind: CommentKind,
  pub comment_id: u64,
  pub reaction_id: u64,
}
