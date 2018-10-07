#![no_std]

extern crate size_format;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
use std::string::ToString;

use size_format::SizeFormatterSI;

#[test]
fn no_std_works() {
    assert_eq!(
        format!("{}B", SizeFormatterSI::new(8_500_000)),
        "8.5MB".to_string()
    );
}
