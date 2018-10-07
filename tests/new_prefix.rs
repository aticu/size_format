extern crate generic_array;
extern crate size_format;

use generic_array::{typenum::U3, GenericArray};
use size_format::{PointSeparated, PrefixType, SizeFormatter};

struct Millimeter;

impl PrefixType for Millimeter {
    type N = U3;

    const PREFIX_SIZE: u32 = 1000;

    fn prefixes() -> GenericArray<&'static str, Self::N> {
        ["m", "", "k"].into()
    }
}

#[test]
fn new_prefix_works() {
    assert_eq!(
        format!(
            "{}m",
            SizeFormatter::<u32, Millimeter, PointSeparated>::new(1)
        ),
        "1mm".to_string()
    );
    assert_eq!(
        format!(
            "{}m",
            SizeFormatter::<u32, Millimeter, PointSeparated>::new(1_000)
        ),
        "1.0m".to_string()
    );
    assert_eq!(
        format!(
            "{}m",
            SizeFormatter::<u32, Millimeter, PointSeparated>::new(1_000_000)
        ),
        "1.0km".to_string()
    );
    assert_eq!(
        format!(
            "{}m",
            SizeFormatter::<u64, Millimeter, PointSeparated>::new(10_000_000_000)
        ),
        "10000.0km".to_string()
    );
}
