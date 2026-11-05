// GitHub logins are case-insensitive, a longer login sharing the prefix
// (`@bot` vs `@bot-two`) is a different user, and an `@` inside a word is an
// email address rather than a mention.
pub fn mentions(
  body: &str,
  login: &str,
) -> bool {
  let is_login_char = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';

  body.match_indices('@').any(|(at, _)| {
    if body[..at].chars().next_back().is_some_and(is_login_char) {
      return false;
    }
    let rest = &body[at + 1..];
    let end = rest.find(|c: char| !is_login_char(c)).unwrap_or(rest.len());
    rest[..end].eq_ignore_ascii_case(login)
  })
}

#[cfg(test)]
mod tests {
  use super::mentions;

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
}
