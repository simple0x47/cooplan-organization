use std::panic;

/// Warning: it is not supposed to work with specific international prefixes such as '00'.
/// The '+' sign must be used.
pub fn is_telephone_valid(telephone: &str) -> bool {
    match panic::catch_unwind(|| match phonenumber::parse(None, telephone.to_string()) {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }) {
        Ok(result) => result.is_ok(),
        Err(_) => false,
    }
}
