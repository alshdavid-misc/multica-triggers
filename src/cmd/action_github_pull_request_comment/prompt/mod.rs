use std::sync::LazyLock;

use handlebars::Handlebars;
use handlebars::handlebars_helper;
use octocrab::models::pulls::Comment as PullRequestComment;
use serde_json::json;

static TEMPLATES: LazyLock<Handlebars<'static>> = LazyLock::new(|| {
  let mut handlebars = Handlebars::new();
  handlebars.set_strict_mode(true);
  handlebars.register_escape_fn(handlebars::no_escape);
  handlebars.register_helper("safe", Box::new(safe));

  for (name, template) in [
    ("top_level", include_str!("./prompt_top_level.hbs")),
    (
      "general_comment",
      include_str!("./prompt_pull_request_general_comment.hbs"),
    ),
    (
      "inline_comment",
      include_str!("./prompt_pull_request_inline_comment.hbs"),
    ),
  ] {
    handlebars
      .register_template_string(name, template)
      .expect("prompt templates are valid");
  }

  handlebars
});

// Comment bodies and diff hunks are reviewer-controlled and are wrapped in XML
// tags, so they must not be able to close or open tags of their own.
handlebars_helper!(safe: |text: str| text
  .replace('&', "&amp;")
  .replace('<', "&lt;")
  .replace('>', "&gt;"));

pub fn prompt_top_level(
  repo: &str,
  branch: &str,
) -> anyhow::Result<String> {
  Ok(TEMPLATES.render("top_level", &json!({ "repo": repo, "branch": branch }))?)
}

pub fn prompt_pull_request_general_comment(body: &str) -> anyhow::Result<String> {
  Ok(TEMPLATES.render("general_comment", &json!({ "body": body }))?)
}

pub fn prompt_pull_request_inline_comment(comment: &PullRequestComment) -> anyhow::Result<String> {
  Ok(TEMPLATES.render(
    "inline_comment",
    &json!({
      "path": comment.path,
      "line": comment.line,
      "start_line": comment.start_line,
      "original_line": comment.original_line,
      "commit_sha": comment.original_commit_id,
      "is_reply": comment.in_reply_to_id.is_some(),
      "thread_id": comment.in_reply_to_id,
      "diff_hunk": comment.diff_hunk,
      "body": comment.body,
    }),
  )?)
}
