//! This module contains all customization options for the formatting.

use generic_array::{typenum::U9, ArrayLength, GenericArray};

/// A trait for marker types that represent decimal separators.
pub trait DecimalSeparator {
    /// The separator to use.
    const SEPARATOR: char;
}

/// Represents a comma separation scheme for numbers (',').
pub struct CommaSeparated;

impl DecimalSeparator for CommaSeparated {
    const SEPARATOR: char = ',';
}

/// Represents a point or dot separation scheme for numbers ('.').
pub struct PointSeparated;

impl DecimalSeparator for PointSeparated {
    const SEPARATOR: char = '.';
}

/// Abstracts over the types of prefixes possible.
pub trait PrefixType {
    /// The number of prefixes in the prefix array.
    type N: ArrayLength<&'static str>;

    /// Returns the size of the prefix used.
    ///
    /// For the metric system for example that would be 1000.
    const PREFIX_SIZE: u32;

    /// Represents the prefixes used by the prefix type.
    fn prefixes() -> GenericArray<&'static str, Self::N>;
}

/// Represents the prefixes used in the SI system of measurements.
pub struct SIPrefixes;

impl PrefixType for SIPrefixes {
    type N = U9;

    const PREFIX_SIZE: u32 = 1000;

    fn prefixes() -> GenericArray<&'static str, Self::N> {
        ["", "k", "M", "G", "T", "P", "E", "Z", "Y"].into()
    }
}

/// Represents the prefixes used for display file sizes using powers of 1024.
pub struct BinaryPrefixes;

impl PrefixType for BinaryPrefixes {
    type N = U9;

    const PREFIX_SIZE: u32 = 1024;

    fn prefixes() -> GenericArray<&'static str, Self::N> {
        ["", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi"].into()
    }
}
