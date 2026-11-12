//! Case-insensitive matching of names.

/// Unicode simple case folding of one character.
pub fn fold_char(c: char) -> char {
    if c.is_ascii() {
        return c.to_ascii_lowercase();
    }
    let mut lower = c.to_lowercase();
    let (Some(lower), None) = (lower.next(), lower.next()) else {
        return c;
    };
    match lower {
        'ſ' => 's',
        'ς' => 'σ',
        'ϐ' => 'β',
        'ϑ' => 'θ',
        'ϕ' => 'φ',
        'ϖ' => 'π',
        'ϰ' => 'κ',
        'ϱ' => 'ρ',
        'ϵ' => 'ε',
        'ẛ' => 'ṡ',
        '\u{1FBE}' => 'ι',
        _ => lower,
    }
}

/// A name with every character case folded, for comparing names.
pub fn fold_case(name: &str) -> String {
    if name.is_ascii() {
        return name.to_ascii_lowercase();
    }
    name.chars().map(fold_char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_simply() {
        assert_eq!(fold_case("Spectral Γ"), "spectral γ");
        assert_eq!(fold_case("ΣΑΣ ς"), "σασ σ");
        assert_eq!(fold_case("ẞ"), "ß");
        assert_eq!(fold_case("İ"), "İ");
    }
}
