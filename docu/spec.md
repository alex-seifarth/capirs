
## Mapping of Data Types between FRANCA, SOME/IP and Rust

|Supp.| Franca Type       |  SOME/IP Type     |    Rust Type | Deployment       |
|-----|-------------------|-------------------|--------------|------------------|
| y   | Int8              | sint8             | i8           |
| y   | UInt8             | uint8             | u8           |
| y   | Int16             | sint16            | i16          |
| y   | UInt16            | uint16            | u16          |
| y   | Int32             | sint32            | i32          |
| y   | UInt32            | uint32            | u32          |
| y   | Int64             | sint64            | i64          |
| y   | UInt64            | uint64            | u64          |
| y   | Boolean           | boolean           | bool         |
| y   | String            | string            | String, &str | encoding, length |
| y   | Float             | float32           | f32          |
| y   | Double            | float64           | f64          |
| y   | ByteBuffer        | array<uint8>      | bytes::Bytes | length           |
| y   | Array of T        | array<T>          | vec<T>, T[]  | length           |
| y   | Struct            | struct            | struct       |
| y   | Union             | union             | enum{ EMPTY, CHOICE1(..), ... }
| y   | Enum              | uintN    (1)      | enum         | width N          |
| ?   | IntegerInterval   | sintN    (2)      | iN/uN        |                  |

NOTE 1: Extension allows specification of 1/2 byte long values with bit-offset
    via deployment (fdepl).
NOTE 2: This is only a restriction of a normal integer type with min/max.
NOTE 3: Deployment may need to specify whether elements are fixed or variable length
      and how long the length indicator shall be.
