//! # size_format
//!
//! This crate provides formatting for sizes.
//!
//! The main goal is to provide easy formatters for data sizes.
//!
//! It provides both binary and SI unit prefixes per default, though more could be added.
//! ```
//! use size_format::{SizeFormatterBinary, SizeFormatterSI};
//!
//! assert_eq!(
//!     format!("{}B", SizeFormatterBinary::new(42 * 1024 * 1024)),
//!     "42.0MiB".to_string()
//! );
//! assert_eq!(
//!     format!("{}B", SizeFormatterSI::new(42_000_000)),
//!     "42.0MB".to_string()
//! );
//! ```
//!
//! The precision can also be specified. Please note that values are always rounded down.
//! ```
//! use size_format::SizeFormatterSI;
//!
//! assert_eq!(
//!     format!("{:.4}B", SizeFormatterSI::new(1_999_999_999)),
//!     "1.9999GB".to_string()
//! );
//! assert_eq!(
//!     format!("{:.0}B", SizeFormatterSI::new(1_999_999_999)),
//!     "1GB".to_string()
//! );
//! ```
//!
//! The presented precision will also never exceed the available precision.
//! ```
//! use size_format::SizeFormatterSI;
//!
//! assert_eq!(
//!     format!("{:.10}B", SizeFormatterSI::new(678)),
//!     "678B".to_string()
//! );
//! assert_eq!(
//!     format!("{:.10}B", SizeFormatterSI::new(1_999)),
//!     "1.999kB".to_string()
//! );
//! ```
//!
//! For more flexibility, use the `SizeFormatter` type directly with the correct type parameters.
//! For example the following code formats a `u16` using binary prefixes and uses a comma as a separator.
//! ```
//! use size_format::{BinaryPrefixes, CommaSeparated, SizeFormatter};
//!
//! assert_eq!(
//!     format!("{:.2}B", SizeFormatter::<u16, BinaryPrefixes, CommaSeparated>::from(65_535u16)),
//!     "63,99KiB".to_string()
//! );
//! ```
//!
//! Although this crate was mainly intended for data sizes, it can also be used for other units.
//!
//! It is also possible to implement the `PrefixType` trait to make your own prefix system.
//! ```
//! use size_format::{PointSeparated, PrefixType, SizeFormatter};
//! use generic_array::{typenum::U3, GenericArray};
//!
//! struct Millimeter;
//!
//! impl PrefixType for Millimeter {
//!     type N = U3;
//!
//!     const PREFIX_SIZE: u32 = 1000;
//!
//!     fn prefixes() -> GenericArray<&'static str, Self::N> {
//!         ["m", "", "k"].into()
//!     }
//! }
//!
//! assert_eq!(
//!     format!("{}m", SizeFormatter::<u32, Millimeter, PointSeparated>::new(1)),
//!     "1mm".to_string()
//! );
//! assert_eq!(
//!     format!("{}m", SizeFormatter::<u32, Millimeter, PointSeparated>::new(1_000)),
//!     "1.0m".to_string()
//! );
//! assert_eq!(
//!     format!("{}m", SizeFormatter::<u32, Millimeter, PointSeparated>::new(1_000_000)),
//!     "1.0km".to_string()
//! );
//! assert_eq!(
//!     format!("{}m", SizeFormatter::<u64, Millimeter, PointSeparated>::new(10_000_000_000)),
//!     "10000.0km".to_string()
//! );
//! ```

#![no_std]
#![warn(missing_docs)]
use core::{
    cmp,
    fmt::{self, Display},
    marker::PhantomData,
};
use num::{integer::Integer, rational::Ratio, traits::cast::FromPrimitive, traits::Pow};

mod config;

pub use self::config::{
    BinaryPrefixes, CommaSeparated, DecimalSeparator, PointSeparated, PrefixType, SIPrefixes,
};

/// The precision to use by default for formatting the numbers.
const DEFAULT_PRECISION: usize = 1;

/// Implements `Display` to format the contained byte size using SI prefixes.
pub type SizeFormatterSI = SizeFormatter<u64, SIPrefixes, PointSeparated>;

/// Implements `Display` to format the contained byte size using binary prefixes.
pub type SizeFormatterBinary = SizeFormatter<u64, BinaryPrefixes, PointSeparated>;

/// Represents a size that can be formatted.
///
/// # Panics
/// - May panic if the `BaseType` is too small for the prefix specified in `Prefix`
///   and the number is being formatted.
pub struct SizeFormatter<BaseType, Prefix, Separator>
where
    BaseType: Clone + Integer + Display + FromPrimitive + Pow<u32, Output = BaseType>,
    Ratio<BaseType>: FromPrimitive,
    Prefix: PrefixType,
    Separator: DecimalSeparator,
{
    /// The number to be formatted.
    num: BaseType,
    _marker: PhantomData<(Prefix, Separator)>,
}

impl<BaseType, Prefix, Separator> SizeFormatter<BaseType, Prefix, Separator>
where
    BaseType: Clone + Integer + Display + FromPrimitive + Pow<u32, Output = BaseType>,
    Ratio<BaseType>: FromPrimitive,
    Prefix: PrefixType,
    Separator: DecimalSeparator,
{
    /// Creates a new size formatter for the given number.
    pub fn new(num: BaseType) -> SizeFormatter<BaseType, Prefix, Separator> {
        SizeFormatter {
            num,
            _marker: PhantomData,
        }
    }

    /// Creates a new size formatter from a compatible number.
    pub fn from<T: Into<BaseType>>(num: T) -> SizeFormatter<BaseType, Prefix, Separator> {
        SizeFormatter {
            num: num.into(),
            _marker: PhantomData,
        }
    }
}

impl<BaseType, Prefix, Separator> Display for SizeFormatter<BaseType, Prefix, Separator>
where
    BaseType: Clone + Integer + Display + FromPrimitive + Pow<u32, Output = BaseType>,
    Ratio<BaseType>: FromPrimitive,
    Prefix: PrefixType,
    Separator: DecimalSeparator,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let max_prefix = Prefix::prefixes().len() - 1;
        let precision = f.precision().unwrap_or(DEFAULT_PRECISION);
        let prefix_size = BaseType::from_u32(Prefix::PREFIX_SIZE)
            .expect("prefix size is too large for number type");

        // Find the right prefix.
        let divisions = cmp::min(int_log(self.num.clone(), prefix_size.clone()), max_prefix);

        // Cap the precision to what makes sense.
        let precision = cmp::min(precision, divisions * 3);

        let ratio = Ratio::<BaseType>::new(self.num.clone(), prefix_size.pow(divisions as u32));

        let format_number = FormatRatio::<BaseType, Separator>::new(ratio);

        write!(
            f,
            "{:.*}{}",
            precision,
            format_number,
            Prefix::prefixes()[divisions]
        )
    }
}

/// Returns the number of times `num` can be divided by `base`.
fn int_log<BaseType>(mut num: BaseType, base: BaseType) -> usize
where
    BaseType: Clone + Integer + Display + FromPrimitive + Pow<u32, Output = BaseType>,
    Ratio<BaseType>: FromPrimitive,
{
    let mut divisions = 0;

    while num >= base {
        num = num / base.clone();
        divisions += 1;
    }

    divisions
}

/// This allows formatting a ratio as a decimal number.
///
/// This is a temporary solution until support for that is added to the `num` crate.
struct FormatRatio<BaseType, Separator>
where
    BaseType: Clone + Integer + Display + FromPrimitive + Pow<u32, Output = BaseType>,
    Ratio<BaseType>: FromPrimitive,
    Separator: DecimalSeparator,
{
    num: Ratio<BaseType>,
    _marker: PhantomData<Separator>,
}

impl<BaseType, Separator> FormatRatio<BaseType, Separator>
where
    BaseType: Clone + Integer + Display + FromPrimitive + Pow<u32, Output = BaseType>,
    Ratio<BaseType>: FromPrimitive,
    Separator: DecimalSeparator,
{
    /// Creates a new format ratio from the number.
    fn new(num: Ratio<BaseType>) -> FormatRatio<BaseType, Separator> {
        FormatRatio {
            num,
            _marker: PhantomData,
        }
    }
}

impl<BaseType, Separator> Display for FormatRatio<BaseType, Separator>
where
    BaseType: Clone + Integer + Display + FromPrimitive + Pow<u32, Output = BaseType>,
    Ratio<BaseType>: FromPrimitive,
    Separator: DecimalSeparator,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.num.trunc())?;
        let precision = f.precision().unwrap_or(DEFAULT_PRECISION);

        if precision > 0 {
            write!(f, "{}", Separator::SEPARATOR)?;
            let mut frac = self.num.fract();

            for _ in 0..precision {
                if frac.is_integer() {
                    // If the fractional part is an integer, we're done and just need more zeroes.
                    write!(f, "0")?;
                } else {
                    // Otherwise print every digit separately.
                    frac = frac * Ratio::from_u64(10).unwrap();
                    write!(f, "{}", frac.trunc())?;
                    frac = frac.fract();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn small_sizes() {
        assert_eq!(format!("{}B", SizeFormatterSI::new(0)), "0B".to_string());
        assert_eq!(format!("{}B", SizeFormatterSI::new(1)), "1B".to_string());
        assert_eq!(
            format!("{}B", SizeFormatterSI::new(999)),
            "999B".to_string()
        );

        assert_eq!(
            format!("{}B", SizeFormatterBinary::new(0)),
            "0B".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterBinary::new(1)),
            "1B".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterBinary::new(999)),
            "999B".to_string()
        );

        assert_eq!(
            format!("{}B", SizeFormatterSI::new(1_000)),
            "1.0kB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterSI::new(55_000)),
            "55.0kB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterSI::new(999_999)),
            "999.9kB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterSI::new(1_000_000)),
            "1.0MB".to_string()
        );

        assert_eq!(
            format!("{}B", SizeFormatterBinary::new(1 * 1024)),
            "1.0KiB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterBinary::new(55 * 1024)),
            "55.0KiB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterBinary::new(999 * 1024 + 1023)),
            "999.9KiB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterBinary::new(1 * 1024 * 1024)),
            "1.0MiB".to_string()
        );
    }

    #[test]
    fn big_sizes() {
        assert_eq!(
            format!("{}B", SizeFormatterSI::new(387_854_348_875)),
            "387.8GB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterSI::new(123_456_789_999_999)),
            "123.4TB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterSI::new(499_999_999_999_999_999)),
            "499.9PB".to_string()
        );
        assert_eq!(
            format!("{}B", SizeFormatterSI::new(1_000_000_000_000_000_000)),
            "1.0EB".to_string()
        );
        assert_eq!(
            format!(
                "{}B",
                SizeFormatter::<u128, SIPrefixes, PointSeparated>::new(
                    1_000_000_000_000_000_000_000
                )
            ),
            "1.0ZB".to_string()
        );
        assert_eq!(
            format!(
                "{}B",
                SizeFormatter::<u128, SIPrefixes, PointSeparated>::new(
                    1_000_000_000_000_000_000_000_000
                )
            ),
            "1.0YB".to_string()
        );
    }

    #[test]
    fn exceeds_yotta() {
        assert_eq!(
            format!(
                "{}B",
                SizeFormatter::<u128, SIPrefixes, PointSeparated>::new(
                    1_000_000_000_000_000_000_000_000_000
                )
            ),
            "1000.0YB".to_string()
        );
        assert_eq!(
            format!(
                "{}B",
                SizeFormatter::<u128, SIPrefixes, PointSeparated>::new(
                    1_000_000_000_000_000_000_000_000_000_000
                )
            ),
            "1000000.0YB".to_string()
        );
    }

    #[test]
    fn precision() {
        assert_eq!(format!("{:.9}B", SizeFormatterSI::new(1)), "1B".to_string());
        assert_eq!(
            format!("{:.0}B", SizeFormatterSI::new(1_111)),
            "1kB".to_string()
        );
        assert_eq!(
            format!("{:.1}B", SizeFormatterSI::new(1_111)),
            "1.1kB".to_string()
        );
        assert_eq!(
            format!("{:.2}B", SizeFormatterSI::new(1_111)),
            "1.11kB".to_string()
        );
        assert_eq!(
            format!("{:.3}B", SizeFormatterSI::new(1_111)),
            "1.111kB".to_string()
        );
        assert_eq!(
            format!("{:.4}B", SizeFormatterSI::new(1_111)),
            "1.111kB".to_string()
        );
        assert_eq!(
            format!("{:.4}B", SizeFormatterSI::new(1_000_100)),
            "1.0001MB".to_string()
        );
        assert_eq!(
            format!("{:.4}B", SizeFormatterSI::new(1_500_000)),
            "1.5000MB".to_string()
        );
        assert_eq!(
            format!("{:.4}B", SizeFormatterSI::new(1_000_000)),
            "1.0000MB".to_string()
        );
    }

    #[test]
    fn configurations() {
        assert_eq!(
            format!(
                "{}B",
                SizeFormatter::<u16, SIPrefixes, CommaSeparated>::new(65_535)
            ),
            "65,5kB".to_string()
        );

        assert_eq!(
            format!(
                "{}B",
                SizeFormatter::<u16, BinaryPrefixes, PointSeparated>::new(65_535)
            ),
            "63.9KiB".to_string()
        );
    }

    #[test]
    fn from() {
        assert_eq!(
            format!("{}B", SizeFormatterSI::from(546_987u32)),
            "546.9kB".to_string()
        );
    }

    #[test]
    #[should_panic(expected = "prefix size is too large")]
    fn incompatile_base_type_fails() {
        assert_eq!(
            format!(
                "{}B",
                SizeFormatter::<u8, SIPrefixes, CommaSeparated>::new(10)
            ),
            "65.5kB".to_string()
        );
    }
}
