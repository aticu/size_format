extern crate size_format;

use size_format::SizeFormatterSI;

#[test]
fn std_works() {
    assert_eq!(
        format!("{}B", SizeFormatterSI::new(8_500_000)),
        "8.5MB".to_string()
    );
}
