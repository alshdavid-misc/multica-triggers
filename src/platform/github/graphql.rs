use octocrab::Octocrab;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;

use super::api::split_repo;

#[derive(Debug, Deserialize)]
pub struct Actor {
  pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct Discussion {
  pub id: String,
  pub title: String,
  pub body: String,
  pub url: String,
  pub author: Option<Actor>,
}

#[derive(Debug, Deserialize)]
pub struct DiscussionComment {
  pub id: String,
  pub body: String,
  pub url: String,
  pub author: Option<Actor>,
}

async fn query<T: DeserializeOwned>(
  client: &Octocrab,
  query: &str,
  variables: serde_json::Value,
) -> anyhow::Result<T> {
  Ok(
    client
      .graphql(&json!({ "query": query, "variables": variables }))
      .await?,
  )
}

pub async fn get_discussion(
  client: &Octocrab,
  repo: &str,
  number: u64,
) -> anyhow::Result<Discussion> {
  #[derive(Deserialize)]
  struct Data {
    repository: Option<Repository>,
  }
  #[derive(Deserialize)]
  struct Repository {
    discussion: Option<Discussion>,
  }

  let (owner, name) = split_repo(repo)?;
  let data: Data = query(
    client,
    "query($owner: String!, $name: String!, $number: Int!) {
      repository(owner: $owner, name: $name) {
        discussion(number: $number) { id title body url author { login } }
      }
    }",
    json!({ "owner": owner, "name": name, "number": number }),
  )
  .await?;

  data
    .repository
    .and_then(|repository| repository.discussion)
    .ok_or_else(|| anyhow::anyhow!("discussion {repo}#{number} not found"))
}

pub async fn get_discussion_comment(
  client: &Octocrab,
  node_id: &str,
) -> anyhow::Result<DiscussionComment> {
  #[derive(Deserialize)]
  struct Data {
    node: Option<DiscussionComment>,
  }

  let data: Data = query(
    client,
    "query($id: ID!) {
      node(id: $id) { ... on DiscussionComment { id body url author { login } } }
    }",
    json!({ "id": node_id }),
  )
  .await?;

  data
    .node
    .ok_or_else(|| anyhow::anyhow!("discussion comment {node_id} not found"))
}

/// Adds the viewer's 👀 reaction to any reactable node (issue, comment, discussion...).
pub async fn add_eyes_reaction(
  client: &Octocrab,
  subject_id: &str,
) -> anyhow::Result<()> {
  query::<serde_json::Value>(
    client,
    "mutation($id: ID!) { addReaction(input: { subjectId: $id, content: EYES }) { clientMutationId } }",
    json!({ "id": subject_id }),
  )
  .await?;
  Ok(())
}

/// Removes the viewer's 👀 reaction from any reactable node.
pub async fn remove_eyes_reaction(
  client: &Octocrab,
  subject_id: &str,
) -> anyhow::Result<()> {
  query::<serde_json::Value>(
    client,
    "mutation($id: ID!) { removeReaction(input: { subjectId: $id, content: EYES }) { clientMutationId } }",
    json!({ "id": subject_id }),
  )
  .await?;
  Ok(())
}
