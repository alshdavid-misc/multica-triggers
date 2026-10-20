use std::sync::LazyLock;

use handlebars::Handlebars;
use handlebars::handlebars_helper;
use serde_json::json;

static TEMPLATES: LazyLock<Handlebars<'static>> = LazyLock::new(|| {
  let mut handlebars = Handlebars::new();
  handlebars.set_strict_mode(true);
  handlebars.register_escape_fn(handlebars::no_escape);
  handlebars.register_helper("safe", Box::new(safe));
  handlebars
    .register_template_string("top_level", include_str!("./prompt_top_level.hbs"))
    .expect("prompt templates are valid");
  handlebars
});

// Comment bodies are reviewer-controlled and are wrapped in XML tags, so they
// must not be able to close or open tags of their own.
handlebars_helper!(safe: |text: str| text
  .replace('&', "&amp;")
  .replace('<', "&lt;")
  .replace('>', "&gt;"));

pub fn prompt_top_level(
  repo: &str,
  branch: &str,
  body: &str,
) -> anyhow::Result<String> {
  Ok(TEMPLATES.render(
    "top_level",
    &json!({ "repo": repo, "branch": branch, "body": body }),
  )?)
}
