#[derive(Debug)]
pub enum UnescapeError {
    TrailingBackslash,
    UnknownEscape(char),
}

/// Convert escape sequences into their raw values.
pub fn unescape_str(s: &str) -> Result<String, UnescapeError> {
    let mut chars = s.chars();
    let mut result = String::with_capacity(s.len());

    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }

        match chars.next() {
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some('\\') => result.push('\\'),
            Some('\'') => result.push('\''),
            Some('\"') => result.push('\"'),
            Some(c) => return Err(UnescapeError::UnknownEscape(c)),
            None => return Err(UnescapeError::TrailingBackslash),
        }
    }

    Ok(result)
}
