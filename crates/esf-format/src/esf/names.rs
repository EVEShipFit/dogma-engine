use esf_data::fold_case;

use super::text::{SIGILS, is_count};

/// Whether `prefix` is `name` with zero or more whole words dropped from the end.
pub(super) fn is_word_prefix(prefix: &str, name: &str) -> bool {
    let prefix = fold_case(prefix);
    let name = fold_case(name);
    let mut words = name.split(' ');
    prefix.split(' ').all(|word| words.next() == Some(word))
}

/// The fewest leading words of `name` that are a word prefix of none of `others`.
pub(super) fn shortest_unique(name: &str, others: &[&str]) -> String {
    let words: Vec<&str> = name.split(' ').collect();
    (1..=words.len())
        .map(|count| words[..count].join(" "))
        .find(|candidate| !others.iter().any(|other| is_word_prefix(candidate, other)))
        .unwrap_or_else(|| name.to_string())
}

pub(super) fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}

fn needs_quotes(name: &str) -> bool {
    let mut words = name.split(' ');
    let first = words.next().unwrap_or_default();
    first.is_empty()
        || first.starts_with(['-', '%'])
        || is_count(first)
        || name
            .split(' ')
            .any(|word| word.is_empty() || word.starts_with(SIGILS))
}

pub(super) fn quote_type(name: &str) -> String {
    match needs_quotes(name) {
        true => quote(name),
        false => name.to_string(),
    }
}

/// The fewest significant digits that read back as the same float, without exponent.
pub(super) fn number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes() {
        assert!(is_word_prefix(
            "unstable 5mn",
            "Unstable 5MN Microwarpdrive"
        ));
        assert!(!is_word_prefix("Unstable 5", "Unstable 5MN Microwarpdrive"));
        assert_eq!(
            shortest_unique(
                "Glorified Unstable X",
                &["Unstable X", "Glorified Decayed X"]
            ),
            "Glorified Unstable"
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(number(-0.0), "0");
        assert_eq!(number(524.0), "524");
        assert_eq!(number(0.1), "0.1");
        assert_eq!(number(1e-7), "0.0000001");
        assert_eq!(number(1e21), "1000000000000000000000");
    }

    #[test]
    fn quoting() {
        assert_eq!(quote_type("Damage Control II"), "Damage Control II");
        assert_eq!(quote_type("Weird/Name II"), "Weird/Name II");
        assert_eq!(quote_type("Weird /Name"), "\"Weird /Name\"");
        assert_eq!(quote_type("-Odd"), "\"-Odd\"");
        assert_eq!(quote_type("5x Thing"), "\"5x Thing\"");
        assert_eq!(quote(r#"Oracle "Blaze""#), r#""Oracle ""Blaze""""#);
    }
}
