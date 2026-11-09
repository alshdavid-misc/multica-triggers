use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use octocrab::Octocrab;
use parking_lot::Mutex;
use tokio::sync::Notify;

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
  pub pending_runs: Arc<Mutex<HashMap<String, PendingRun>>>,
  pub poll_runs: Arc<Notify>,
  pub cleanup_lock: Arc<tokio::sync::Mutex<()>>,
}

pub type RunCleanup =
  Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> + Send>;

pub struct PendingRun {
  pub run_id: String,
  pub autopilot_id: String,
  pub cleanup: Option<RunCleanup>,
  pub cancel_requested: bool,
}

pub fn run_key(run_id: &str) -> String {
  format!("run:{run_id}")
}

pub fn pull_request_key(
  repo: &str,
  number: u64,
) -> String {
  format!("pull_request:{repo}#{number}")
}
