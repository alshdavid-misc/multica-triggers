use std::collections::HashMap;
use std::sync::Mutex;

use octocrab::Octocrab;

use crate::context::PendingRun;
use crate::context::PendingRunCleanup;
use crate::platform::github;
use crate::platform::multica::WebhookResponse;

/// Marks the GitHub node (issue, comment or discussion) with 👀 once Multica
/// accepts the webhook, and tracks the run so the reaction is removed when it
/// finishes.
pub async fn acknowledge(
  github: &Octocrab,
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
    pending_runs.lock().unwrap().insert(
      run_id,
      PendingRun {
        autopilot_id,
        cleanup: PendingRunCleanup::RemoveEyesReaction { subject_id },
      },
    );
  }

  Ok(())
}
