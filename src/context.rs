use std::sync::Arc;

use octocrab::Octocrab;

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
  pub multica_pull_request_sync_webhook: Option<String>,
  pub github: Arc<Octocrab>,
}
