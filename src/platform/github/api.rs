use octocrab::Octocrab;
use octocrab::models::CommentId;
use octocrab::models::ReactionId;
use octocrab::models::UserId;
use octocrab::models::issues::Comment;
use octocrab::models::issues::Issue;
use octocrab::models::pulls::Comment as PullRequestComment;
use octocrab::models::pulls::PullRequest;
use octocrab::models::reactions::ReactionContent;

pub async fn get_issue(
  client: &Octocrab,
  repo: &str,
  number: u64,
) -> anyhow::Result<Issue> {
  let (owner, name) = split_repo(repo)?;
  Ok(client.issues(owner, name).get(number).await?)
}

pub async fn get_issue_comment(
  client: &Octocrab,
  repo: &str,
  comment_id: u64,
) -> anyhow::Result<Comment> {
  let (owner, name) = split_repo(repo)?;
  Ok(
    client
      .issues(owner, name)
      .get_comment(CommentId(comment_id))
      .await?,
  )
}

pub async fn get_pull_request(
  client: &Octocrab,
  repo: &str,
  number: u64,
) -> anyhow::Result<PullRequest> {
  let (owner, name) = split_repo(repo)?;
  Ok(client.pulls(owner, name).get(number).await?)
}

pub async fn get_pr_comment(
  client: &Octocrab,
  repo: &str,
  comment_id: u64,
) -> anyhow::Result<PullRequestComment> {
  let (owner, name) = split_repo(repo)?;
  Ok(
    client
      .pulls(owner, name)
      .comment(CommentId(comment_id))
      .get()
      .await?,
  )
}

#[derive(Debug, Clone, Copy)]
pub enum ReactionTarget {
  Issue(u64),
  IssueComment(u64),
  ReviewComment(u64),
}

pub async fn add_reaction(
  client: &Octocrab,
  repo: &str,
  target: ReactionTarget,
  content: ReactionContent,
) -> anyhow::Result<u64> {
  let (owner, name) = split_repo(repo)?;
  let reaction = match target {
    ReactionTarget::Issue(number) => {
      client
        .issues(owner, name)
        .create_reaction(number, content)
        .await?
    }
    ReactionTarget::IssueComment(comment_id) => {
      client
        .issues(owner, name)
        .create_comment_reaction(CommentId(comment_id), content)
        .await?
    }
    ReactionTarget::ReviewComment(comment_id) => {
      client
        .pulls(owner, name)
        .create_comment_reaction(CommentId(comment_id), content)
        .await?
    }
  };
  Ok(reaction.id.into_inner())
}

pub async fn remove_reaction(
  client: &Octocrab,
  repo: &str,
  target: ReactionTarget,
  reaction_id: u64,
) -> anyhow::Result<()> {
  let (owner, name) = split_repo(repo)?;
  let reaction_id = ReactionId(reaction_id);
  match target {
    ReactionTarget::Issue(number) => {
      client
        .issues(owner, name)
        .delete_reaction(number, reaction_id)
        .await?
    }
    ReactionTarget::IssueComment(comment_id) => {
      client
        .issues(owner, name)
        .delete_comment_reaction(CommentId(comment_id), reaction_id)
        .await?
    }
    ReactionTarget::ReviewComment(comment_id) => {
      client
        .pulls(owner, name)
        .delete_comment_reaction(CommentId(comment_id), reaction_id)
        .await?
    }
  };
  Ok(())
}

pub async fn get_user_login(
  client: &Octocrab,
  user_id: u64,
) -> anyhow::Result<String> {
  Ok(client.users_by_id(UserId(user_id)).profile().await?.login)
}

/// Add a label to an issue.
pub async fn add_label(
  client: &Octocrab,
  issue: &Issue,
  label: &str,
) -> anyhow::Result<()> {
  let (owner, name) = repo_of(issue)?;
  client
    .issues(owner, name)
    .add_labels(issue.number, &[label.to_string()])
    .await?;
  Ok(())
}

fn split_repo(repo: &str) -> anyhow::Result<(&str, &str)> {
  repo
    .split_once('/')
    .ok_or_else(|| anyhow::anyhow!("invalid repo name, expected owner/name: {repo}"))
}

/// Recover `(owner, name)` from the issue's `repository_url`
/// (`https://api.github.com/repos/{owner}/{name}`).
fn repo_of(issue: &Issue) -> anyhow::Result<(String, String)> {
  let mut segments = issue
    .repository_url
    .path_segments()
    .ok_or_else(|| anyhow::anyhow!("issue has no repository url path"))?;

  match (segments.next(), segments.next(), segments.next()) {
    (Some("repos"), Some(owner), Some(name)) => Ok((owner.to_string(), name.to_string())),
    _ => anyhow::bail!("unexpected repository url: {}", issue.repository_url),
  }
}
