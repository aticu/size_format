# size_format

This is a rust crate providing formatting for sizes using prefixes.

For example 4000 bytes could be formatted as 4.0kB.

The main goal is to provide easy formatters for data sizes.

It provides both binary and SI unit prefixes per default, though more could be added.

```rust
use size_format::{SizeFormatterBinary, SizeFormatterSI};

assert_eq!(
    format!("{}B", SizeFormatterBinary::new(42 * 1024 * 1024)),
    "42.0MiB".to_string()
);
assert_eq!(
    format!("{}B", SizeFormatterSI::new(42_000_000)),
    "42.0MB".to_string()
);
```

The precision can also be specified. Please note that values are always rounded down.

```rust
use size_format::SizeFormatterSI;

assert_eq!(
    format!("{:.4}B", SizeFormatterSI::new(1_999_999_999)),
    "1.9999GB".to_string()
);
assert_eq!(
    format!("{:.0}B", SizeFormatterSI::new(1_999_999_999)),
    "1GB".to_string()
);
```

The presented precision will also never exceed the available precision.

```rust
use size_format::SizeFormatterSI;

assert_eq!(
    format!("{:.10}B", SizeFormatterSI::new(678)),
    "678B".to_string()
);
assert_eq!(
    format!("{:.10}B", SizeFormatterSI::new(1_999)),
    "1.999kB".to_string()
);
```

For more flexibility, use the `SizeFormatter` type directly with the correct type parameters.
For example the following code formats a `u16` using binary prefixes and uses a comma as a separator.

```rust
use size_format::{BinaryPrefixes, CommaSeparated, SizeFormatter};

assert_eq!(
    format!("{:.2}B", SizeFormatter::<u16, BinaryPrefixes, CommaSeparated>::from(65_535u16)),
    "63,99KiB".to_string()
);
```

Although this crate was mainly intended for data sizes, it can also be used for other units.

It is also possible to implement the `PrefixType` trait to make your own prefix system.

```rust
use size_format::{PointSeparated, PrefixType, SizeFormatter};
use generic_array::{typenum::U3, GenericArray};

struct Millimeter;

impl PrefixType for Millimeter {
    type N = U3;

    const PREFIX_SIZE: u32 = 1000;

    fn prefixes() -> GenericArray<&'static str, Self::N> {
        ["m", "", "k"].into()
    }
}

assert_eq!(
    format!("{}m", SizeFormatter::<u32, Millimeter, PointSeparated>::new(1)),
    "1mm".to_string()
);
assert_eq!(
    format!("{}m", SizeFormatter::<u32, Millimeter, PointSeparated>::new(1_000)),
    "1.0m".to_string()
);
assert_eq!(
    format!("{}m", SizeFormatter::<u32, Millimeter, PointSeparated>::new(1_000_000)),
    "1.0km".to_string()
);
assert_eq!(
    format!("{}m", SizeFormatter::<u64, Millimeter, PointSeparated>::new(10_000_000_000)),
    "10000.0km".to_string()
);
```