use std::sync::Arc;

use octocrab::Octocrab;

#[derive(Clone)]
pub struct AppContext {
  pub multica_url: String,
  pub multica_token: String,
  pub multica_workspace_id: String,
  pub multica_project_id: Option<String>,
  pub multica_agent: String,
  pub github_token: String,
  pub github_issue_label: String,
  pub github_user_reply_list: String,
  pub github: Arc<Octocrab>,
}
