fn is_login_char(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

// GitHub logins are case-insensitive, a longer login sharing the prefix
// (`@bot` vs `@bot-two`) is a different user, and an `@` inside a word is an
// email address rather than a mention.
fn mention_ends<'a>(
  body: &'a str,
  login: &'a str,
) -> impl Iterator<Item = usize> + 'a {
  body.match_indices('@').filter_map(move |(at, _)| {
    if body[..at].chars().next_back().is_some_and(is_login_char) {
      return None;
    }
    let start = at + 1;
    let rest = &body[start..];
    let end = rest.find(|c: char| !is_login_char(c)).unwrap_or(rest.len());
    rest[..end]
      .eq_ignore_ascii_case(login)
      .then_some(start + end)
  })
}

pub fn mentions(
  body: &str,
  login: &str,
) -> bool {
  mention_ends(body, login).next().is_some()
}

pub fn mentions_command(
  body: &str,
  login: &str,
  command: &str,
) -> bool {
  mention_ends(body, login).any(|end| {
    let rest = &body[end..];
    let trimmed = rest.trim_start_matches([' ', '\t']);
    if trimmed.len() == rest.len() {
      return false;
    }
    let Some(rest) = trimmed
      .strip_prefix('/')
      .and_then(|rest| rest.strip_prefix(command))
    else {
      return false;
    };
    !rest.starts_with(is_login_char)
  })
}

#[cfg(test)]
mod tests {
  use super::mentions;
  use super::mentions_command;

  #[test]
  fn matches_mention() {
    assert!(mentions("@bot please fix", "bot"));
    assert!(mentions("hey @Bot, thoughts?", "bot"));
    assert!(mentions("@sentinal-one_psgit asd", "sentinal-one_psgit"));
  }

  #[test]
  fn ignores_non_mentions() {
    assert!(!mentions("@bot-two please fix", "bot"));
    assert!(!mentions("@bot_psgit please fix", "bot"));
    assert!(!mentions("mail me@bot.com", "bot"));
    assert!(!mentions("no mention here", "bot"));
  }

  #[test]
  fn matches_command() {
    assert!(mentions_command("@bot /review", "bot", "review"));
    assert!(mentions_command("@Bot  /review please", "bot", "review"));
    assert!(mentions_command("thanks\n@bot /review\n", "bot", "review"));
    assert!(mentions_command(
      "@bot please\n@bot /review",
      "bot",
      "review"
    ));
  }

  #[test]
  fn ignores_non_commands() {
    assert!(!mentions_command("@bot please review", "bot", "review"));
    assert!(!mentions_command("@bot/review", "bot", "review"));
    assert!(!mentions_command("@bot /reviewer", "bot", "review"));
    assert!(!mentions_command("@bot-two /review", "bot", "review"));
    assert!(!mentions_command("@bot\n/review", "bot", "review"));
    assert!(!mentions_command("/review @bot", "bot", "review"));
  }
}
