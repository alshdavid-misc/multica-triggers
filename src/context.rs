use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use octocrab::Octocrab;

#[derive(Clone)]
pub struct AppContext {
  pub multica_url: String,
  pub multica_token: String,
  pub multica_workspace_id: String,
  pub github_webhook_secret: String,
  pub github_user_reply_list: Option<Vec<u64>>,
  pub github_machine_user_id: u64,
  pub multica_discussion_mention_webhook: Option<String>,
  pub multica_issue_mention_webhook: Option<String>,
  pub multica_pull_request_comment_webhook: Option<String>,
  pub multica_pull_request_sync_webhook: Option<String>,
  pub github: Arc<Octocrab>,
  /// Autopilot runs, keyed by run id, whose GitHub node still carries a 👀 reaction.
  pub pending_runs: Arc<Mutex<HashMap<String, PendingRun>>>,
}

#[derive(Debug, Clone)]
pub struct PendingRun {
  pub autopilot_id: String,
  /// GraphQL node id of the issue, comment or discussion that was reacted to.
  pub subject_id: String,
}
