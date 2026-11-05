use std::collections::HashMap;
use std::sync::Mutex;

use octocrab::Octocrab;
use octocrab::models::reactions::ReactionContent;

use crate::context::PendingRun;
use crate::platform::github;
use crate::platform::github::ReactionTarget;
use crate::platform::multica::WebhookResponse;

/// Marks the GitHub issue or comment with 👀 once Multica accepts the webhook,
/// and tracks the run so the reaction is removed when it finishes.
pub async fn acknowledge(
  github: &Octocrab,
  pending_runs: &Mutex<HashMap<String, PendingRun>>,
  repo: &str,
  target: ReactionTarget,
  response: WebhookResponse,
) -> anyhow::Result<()> {
  if response.status != "accepted" {
    eprintln!(
      "webhook not accepted ({}) for {repo} {target:?}",
      response.status
    );
    return Ok(());
  }

  let reaction_id = github::add_reaction(github, repo, target, ReactionContent::Eyes).await?;

  if let (Some(run_id), Some(autopilot_id)) = (response.run_id, response.autopilot_id) {
    pending_runs.lock().unwrap().insert(
      run_id,
      PendingRun {
        autopilot_id,
        repo: repo.to_owned(),
        target,
        reaction_id,
      },
    );
  }

  Ok(())
}
