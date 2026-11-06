use octocrab::Octocrab;
use octocrab::models::CommentId;
use octocrab::models::UserId;
use octocrab::models::issues::Comment;
use octocrab::models::issues::Issue;
use octocrab::models::pulls::Comment as PullRequestComment;
use octocrab::models::pulls::PullRequest;

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

pub async fn create_issue_comment(
  client: &Octocrab,
  repo: &str,
  number: u64,
  body: &str,
) -> anyhow::Result<Comment> {
  let (owner, name) = split_repo(repo)?;
  Ok(
    client
      .issues(owner, name)
      .create_comment(number, body)
      .await?,
  )
}

pub async fn delete_issue_comment(
  client: &Octocrab,
  repo: &str,
  comment_id: u64,
) -> anyhow::Result<()> {
  let (owner, name) = split_repo(repo)?;
  Ok(
    client
      .issues(owner, name)
      .delete_comment(CommentId(comment_id))
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

pub async fn get_user_login(
  client: &Octocrab,
  user_id: u64,
) -> anyhow::Result<String> {
  Ok(client.users_by_id(UserId(user_id)).profile().await?.login)
}

pub(super) fn split_repo(repo: &str) -> anyhow::Result<(&str, &str)> {
  repo
    .split_once('/')
    .ok_or_else(|| anyhow::anyhow!("invalid repo name, expected owner/name: {repo}"))
}
