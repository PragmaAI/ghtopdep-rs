use crate::dependent::convert_stars_to_number;

#[test]
fn test_convert_stars_to_number() {
    assert_eq!(convert_stars_to_number("100"), 100.0);
    assert_eq!(convert_stars_to_number("1.2k"), 1200.0);
    assert_eq!(convert_stars_to_number("1,234"), 1234.0);
    assert_eq!(convert_stars_to_number("N/A"), -1.0);
    assert_eq!(convert_stars_to_number(""), 0.0);
    assert_eq!(convert_stars_to_number("invalid"), 0.0);
} 