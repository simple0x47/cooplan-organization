use celes::Country;
use std::str::FromStr;

/// Validate whether or not a country is valid by passing a two letter country code.
pub fn is_country_code_valid(country_code: &str) -> bool {
    Country::from_str(country_code).is_ok()
}

#[cfg(test)]
#[test]
fn detect_incorrect_country_codes() {
    assert!(!is_country_code_valid("XX"));
    assert!(!is_country_code_valid("YY"));
}

#[test]
fn detect_correct_country_codes() {
    assert!(is_country_code_valid("ES"));
    assert!(is_country_code_valid("RO"));
    assert!(is_country_code_valid("IT"));
    assert!(is_country_code_valid("DE"));
    assert!(is_country_code_valid("DK"));
}
