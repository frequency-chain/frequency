use crate::utils::validator::HandleValidator;

#[test]
fn test_is_reserved_handle_happy_path() {
    let reserved_handles: Vec<&str> = vec!["admin", "everyone", "all"];
    let handle_validator = HandleValidator::new();

    for handle in reserved_handles {
        assert!(handle_validator.is_reserved_handle(handle));
    }
}

#[test]
fn test_is_reserved_handle_negative() {
    let handles: Vec<&str> = vec!["albert", "coca_cola", "freemont"];
    let handle_validator = HandleValidator::new();
    for handle in handles {
        assert!(!handle_validator.is_reserved_handle(handle));
    }
}

#[test]
fn test_contains_blocked_characters_happy_path() {
    let handles: Vec<&str> = vec!["@lbert", "coca:cola", "#freemont", "charles.darwin", "`String`"];
    let handle_validator = HandleValidator::new();
    for handle in handles {
        assert!(handle_validator.contains_blocked_characters(handle));
    }
}

#[test]
fn test_contains_blocked_characters_negative() {
    let handles: Vec<&str> = vec!["albert", "coca_cola", "freemont", "charles-darwin", "'String'"];
    let handle_validator = HandleValidator::new();
    for handle in handles {
        assert!(!handle_validator.contains_blocked_characters(handle));
    }
}
