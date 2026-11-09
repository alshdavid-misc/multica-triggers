use std::collections::HashMap;
use std::sync::Arc;

use octocrab::Octocrab;
use parking_lot::Mutex;

use crate::context::PendingRun;
use crate::context::run_key;
use crate::platform::github;
use crate::platform::multica::WebhookResponse;

/// Marks the GitHub node (issue, comment or discussion) with 👀 once Multica
/// accepts the webhook, and tracks the run so the reaction is removed when it
/// finishes.
pub async fn acknowledge(
  github: &Arc<Octocrab>,
  pending_runs: &Mutex<HashMap<String, PendingRun>>,
  subject_id: String,
  response: WebhookResponse,
) -> anyhow::Result<()> {
  if response.status != "accepted" {
    eprintln!(
      "webhook not accepted ({}) for {subject_id}",
      response.status
    );
    return Ok(());
  }

  github::add_eyes_reaction(github, &subject_id).await?;

  if let (Some(run_id), Some(autopilot_id)) = (response.run_id, response.autopilot_id) {
    let github = github.clone();

    pending_runs.lock().insert(
      run_key(&run_id),
      PendingRun {
        run_id,
        autopilot_id,
        cleanup: Some(Box::new(move || {
          Box::pin(async move { github::remove_eyes_reaction(&github, &subject_id).await })
        })),
        cancel_requested: false,
      },
    );
  }

  Ok(())
}
