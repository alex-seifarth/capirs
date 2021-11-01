use bytes;
use std;

/// Byte order of a byte sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteOrder {
    /// Byte order with least significant byte coming first.
    LittleEndian,

    /// Byte order with most significant byte coming first.
    BigEndian,
}

/// Parses a single byte from the input as SOME/IP boolean value.
/// SOME/IP booleans are encoding its true value in the lowest bit as 0 = false and 1 = true.
/// The remaining bits must be ignored.
pub fn boolean<Input>() -> impl Fn(Input) -> nom::IResult<Input, bool>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf
{
    move |i: Input| {
        let (rem, val) = uint8()(i)?;
        Ok((rem, (val & 0x01) != 0))
    }
}

/// Parses a single byte from the input as SOME/IP uint8 value.
pub fn uint8<Input>() -> impl Fn(Input) -> nom::IResult<Input, u8>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf
{
    assert_eq!(std::mem::size_of::<u8>(), 1);
    move |i: Input| {
        let (rem, val) = nom::bytes::complete::take(1usize)(i)?;
        Ok((rem, val.clone().get_u8()))
    }
}

/// Parses a single byte from the input as SOME/IP sint8 value.
pub fn sint8<Input>() -> impl Fn(Input) -> nom::IResult<Input, i8>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf
{
    assert_eq!(std::mem::size_of::<i8>(), 1);
    move |i: Input| {
        let (rem, val) = nom::bytes::complete::take(1usize)(i)?;
        Ok((rem, val.clone().get_i8()))
    }
}

/// Parses 2 bytes from the input as SOME/IP uint16 value
pub fn uint16<Input>(data_byte_order: ByteOrder) -> impl Fn(Input) -> nom::IResult<Input, u16>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf,
{
    assert_eq!(std::mem::size_of::<u16>(), 2);
    move |i: Input| {
        let (rem, val) = nom::bytes::complete::take(2usize)(i)?;
        let mut valw = val.clone();
        match data_byte_order {
            ByteOrder::LittleEndian => Ok((rem, valw.get_u16_le())),
            ByteOrder::BigEndian => Ok((rem, valw.get_u16())),
        }
    }
}

/// Parses 2 bytes from the input as SOME/IP sint16 value
pub fn sint16<Input>(data_byte_order: ByteOrder) -> impl Fn(Input) -> nom::IResult<Input, i16>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf,
{
    assert_eq!(std::mem::size_of::<i16>(), 2);
    move |i: Input| {
        let (rem, val) = nom::bytes::complete::take(2usize)(i)?;
        let mut valw = val.clone();
        match data_byte_order {
            ByteOrder::LittleEndian => Ok((rem, valw.get_i16_le())),
            ByteOrder::BigEndian => Ok((rem, valw.get_i16())),
        }
    }
}

/// Parses 4 bytes from the input as SOME/IP uint32 value
pub fn uint32<Input>(data_byte_order: ByteOrder) -> impl Fn(Input) -> nom::IResult<Input, u32>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf,
{
    assert_eq!(std::mem::size_of::<u32>(), 4);
    move |i: Input| {
        let (rem, val) = nom::bytes::complete::take(4usize)(i)?;
        let mut valw = val.clone();
        match data_byte_order {
            ByteOrder::LittleEndian => Ok((rem, valw.get_u32_le())),
            ByteOrder::BigEndian => Ok((rem, valw.get_u32())),
        }
    }
}

/// Parses 4 bytes from the input as SOME/IP sint32 value
pub fn sint32<Input>(data_byte_order: ByteOrder) -> impl Fn(Input) -> nom::IResult<Input, i32>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf,
{
    assert_eq!(std::mem::size_of::<i32>(), 4);
    move |i: Input| {
        let (rem, val) = nom::bytes::complete::take(4usize)(i)?;
        let mut valw = val.clone();
        match data_byte_order {
            ByteOrder::LittleEndian => Ok((rem, valw.get_i32_le())),
            ByteOrder::BigEndian => Ok((rem, valw.get_i32())),
        }
    }
}

/// Parses 8 bytes from the input as SOME/IP uint64 value
pub fn uint64<Input>(data_byte_order: ByteOrder) -> impl Fn(Input) -> nom::IResult<Input, u64>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf,
{
    assert_eq!(std::mem::size_of::<u64>(), 8);
    move |i: Input| {
        let (rem, val) = nom::bytes::complete::take(8usize)(i)?;
        let mut valw = val.clone();
        match data_byte_order {
            ByteOrder::LittleEndian => Ok((rem, valw.get_u64_le())),
            ByteOrder::BigEndian => Ok((rem, valw.get_u64())),
        }
    }
}

/// Parses 8 bytes from the input as SOME/IP sint64 value
pub fn sint64<Input>(data_byte_order: ByteOrder) -> impl Fn(Input) -> nom::IResult<Input, i64>
    where Input: nom::InputIter + nom::InputTake + Clone + bytes::Buf
{
    assert_eq!(std::mem::size_of::<i64>(), 8);
    move |i: Input| {
        let (rem, val) = nom::bytes::complete::take(8usize)(i)?;
        let mut valw = val.clone();
        match data_byte_order {
            ByteOrder::LittleEndian => Ok((rem, valw.get_i64_le())),
            ByteOrder::BigEndian => Ok((rem, valw.get_i64())),
        }
    }
}

/// Encoding of text into SOME/IP string on-the-wire format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringEncoding {
    Utf8,
    Utf16LE,
    Utf16BE,
}

/// Size of length indicators in SOME/IP on-the-wire format
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LengthSize {
    /// Length field with 1 byte size.
    Length1 = 1,

    /// Length field with 2 byte size.
    Length2 = 2,

    /// Length field with 4 byte size.
    Length4 = 4,
}

/// Parses a variably sized string from the input bytes.
/// PRS_SOMEIP_00089, PRS_SOMEIP_00090, PRS_SOMEIP_00091, PRS_SOMEIP_00092
pub fn string(endian: ByteOrder, encoding: StringEncoding, length_size: LengthSize, max_length: usize)
              -> impl Fn(&[u8]) -> nom::IResult<&[u8], String>
{
    move |i: &[u8]| {
        let (rem, length) = match length_size {
            LengthSize::Length1 => { let (r, v) = uint8()(i)?; (r, v as usize) },
            LengthSize::Length2 => { let (r, v) = uint16(endian)(i)?; (r, v as usize) },
            LengthSize::Length4 => { let (r, v) = uint32(endian)(i)?; (r, v as usize) },
        };
        if length > max_length {
            return Err(nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::LengthValue)));
        }
        let (rem2, data) = nom::bytes::complete::take(length)(rem)?;
        match encoding {
            StringEncoding::Utf8 => decode_utf8_data(data, i, rem2),
            StringEncoding::Utf16BE => todo!(),
            StringEncoding::Utf16LE => todo!(),
        }
    }
}

fn decode_utf8_data<'a>(data: &'a[u8], orig: &'a[u8], remains: &'a[u8]) -> nom::IResult<&'a[u8], String> {
    let mut raw_str = Vec::from(data);
    let last_byte = raw_str.last();
    if last_byte.is_none() {
        return Err(nom::Err::Error(nom::error::Error::new(orig, nom::error::ErrorKind::LengthValue)));
    }
    if let Some(end_byte) = last_byte {
        if *end_byte != 0x00 {
            return Err(nom::Err::Error(nom::error::Error::new(orig, nom::error::ErrorKind::Verify)));
        }
    }
    let _ = raw_str.pop(); // remove trailing terminator 0x00 - otherwise the result string will have it appended
    match String::from_utf8(raw_str) {
        Ok(s) => Ok((remains, s)),
        Err(_) => Err(nom::Err::Error(nom::error::Error::new(orig, nom::error::ErrorKind::Fail)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bool() {
        assert_eq!(boolean()(&b"\x00\x11\x12\xf8\x83"[..]), Ok((&b"\x11\x12\xf8\x83"[..], false)));
        assert_eq!(boolean()(&b"\x01\xf8\x83"[..]), Ok((&b"\xf8\x83"[..], true)));
        assert_eq!(boolean()(&b"\xf2\xf8\x83"[..]), Ok((&b"\xf8\x83"[..], false)));
        assert_eq!(boolean()(&b"\x83\xf8\x83"[..]), Ok((&b"\xf8\x83"[..], true)));
        assert_eq!(boolean()(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_uint8() {
        assert_eq!(uint8()(&b"\x00\x11\x12\xf8\x83"[..]), Ok((&b"\x11\x12\xf8\x83"[..], 0x00 as u8)));
        assert_eq!(uint8()(&b"\x12\xf8\x83"[..]), Ok((&b"\xf8\x83"[..], 0x12 as u8)));
        assert_eq!(uint8()(&b"\xf2\xf8\x83"[..]), Ok((&b"\xf8\x83"[..], 0xf2 as u8)));
        assert_eq!(uint8()(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_sint8() {
        assert_eq!(sint8()(&b"\x00\x11\x12\xf8\x83"[..]), Ok((&b"\x11\x12\xf8\x83"[..], 0 as i8)));
        assert_eq!(sint8()(&b"\x12\xf8\x83"[..]), Ok((&b"\xf8\x83"[..], 18 as i8)));
        assert_eq!(sint8()(&b"\xf2\xf8\x83"[..]), Ok((&b"\xf8\x83"[..], -14 as i8)));
        assert_eq!(sint8()(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_uint16_be() {
        assert_eq!(uint16(ByteOrder::BigEndian)(&b"\x00\x00\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 0 as u16)));
        assert_eq!(uint16(ByteOrder::BigEndian)(&b"\x00\x04\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 4 as u16)));
        assert_eq!(uint16(ByteOrder::BigEndian)(&b"\x80\x04\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 32772 as u16)));
        assert_eq!(uint16(ByteOrder::BigEndian)(&b"\x10\xf4\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 4340 as u16)));
        assert_eq!(uint16(ByteOrder::BigEndian)(&b"\x04\x00\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 1024 as u16)));

        assert_eq!(uint16(ByteOrder::BigEndian)(&b""[..]), Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
        assert_eq!(uint16(ByteOrder::BigEndian)(&b"\x01"[..]), Err(nom::Err::Error(nom::error::Error::new(&b"\x01"[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_uint16_le() {
        assert_eq!(uint16(ByteOrder::LittleEndian)(&b"\x00\x00\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 0 as u16)));
        assert_eq!(uint16(ByteOrder::LittleEndian)(&b"\x00\x04\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 1024 as u16)));
        assert_eq!(uint16(ByteOrder::LittleEndian)(&b"\x80\x04\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 1152 as u16)));
        assert_eq!(uint16(ByteOrder::LittleEndian)(&b"\x10\xf4\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 62480 as u16)));
        assert_eq!(uint16(ByteOrder::LittleEndian)(&b"\x04\x00\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 4 as u16)));

        assert_eq!(uint16(ByteOrder::LittleEndian)(&b""[..]), Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
        assert_eq!(uint16(ByteOrder::LittleEndian)(&b"\x01"[..]), Err(nom::Err::Error(nom::error::Error::new(&b"\x01"[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_sint16_be() {
        assert_eq!(sint16(ByteOrder::BigEndian)(&b"\x00\x00\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 0 as i16)));
        assert_eq!(sint16(ByteOrder::BigEndian)(&b"\x00\x04\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 4 as i16)));
        assert_eq!(sint16(ByteOrder::BigEndian)(&b"\x80\x04\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], -32764 as i16)));
        assert_eq!(sint16(ByteOrder::BigEndian)(&b"\x10\xf4\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 4340 as i16)));
        assert_eq!(sint16(ByteOrder::BigEndian)(&b"\x04\x00\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 1024 as i16)));

        assert_eq!(sint16(ByteOrder::BigEndian)(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
        assert_eq!(sint16(ByteOrder::BigEndian)(&b"\x01"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01"[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_sint16_le() {
        assert_eq!(sint16(ByteOrder::LittleEndian)(&b"\x00\x00\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 0 as i16)));
        assert_eq!(sint16(ByteOrder::LittleEndian)(&b"\x00\x04\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 1024 as i16)));
        assert_eq!(sint16(ByteOrder::LittleEndian)(&b"\x80\x04\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 1152 as i16)));
        assert_eq!(sint16(ByteOrder::LittleEndian)(&b"\x10\xf4\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], -3056 as i16)));
        assert_eq!(sint16(ByteOrder::LittleEndian)(&b"\x04\x00\x00\x04\x10\x12"[..]), Ok((&b"\x00\x04\x10\x12"[..], 4 as i16)));

        assert_eq!(sint16(ByteOrder::LittleEndian)(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
        assert_eq!(sint16(ByteOrder::LittleEndian)(&b"\x01"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01"[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_uint32_be() {
        assert_eq!(uint32(ByteOrder::BigEndian)(&b"\x00\x00\x00\x00\x10\x12"[..]), Ok((&b"\x10\x12"[..], 0 as u32)));
        assert_eq!(uint32(ByteOrder::BigEndian)(&b"\x00\x00\x00\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 4 as u32)));
        assert_eq!(uint32(ByteOrder::BigEndian)(&b"\x80\x04\x00\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 2147745796 as u32)));
        assert_eq!(uint32(ByteOrder::BigEndian)(&b"\x00\x00\x10\xf4\x10\x12"[..]), Ok((&b"\x10\x12"[..], 4340 as u32)));
        assert_eq!(uint32(ByteOrder::BigEndian)(&b"\xff\x01\x89\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 4278290692 as u32)));

        assert_eq!(uint32(ByteOrder::BigEndian)(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
        assert_eq!(uint32(ByteOrder::BigEndian)(&b"\x01"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01"[..], nom::error::ErrorKind::Eof))));
        assert_eq!(uint32(ByteOrder::BigEndian)(&b"\x01\x02"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01\x02"[..], nom::error::ErrorKind::Eof))));
        assert_eq!(uint32(ByteOrder::BigEndian)(&b"\x01\x02\x03"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01\x02\x03"[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_uint32_le() {
        assert_eq!(uint32(ByteOrder::LittleEndian)(&b"\x00\x00\x00\x00\x10\x12"[..]), Ok((&b"\x10\x12"[..], 0 as u32)));
        assert_eq!(uint32(ByteOrder::LittleEndian)(&b"\x00\x00\x00\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 1 << 26 as u32)));
        assert_eq!(uint32(ByteOrder::LittleEndian)(&b"\x80\x04\x00\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 67110016 as u32)));
        assert_eq!(uint32(ByteOrder::LittleEndian)(&b"\x00\x00\x10\xf4\x10\x12"[..]), Ok((&b"\x10\x12"[..], 4094689280 as u32)));
        assert_eq!(uint32(ByteOrder::LittleEndian)(&b"\xff\x01\x89\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 76087807 as u32)));

        assert_eq!(uint32(ByteOrder::LittleEndian)(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
        assert_eq!(uint32(ByteOrder::LittleEndian)(&b"\x01"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01"[..], nom::error::ErrorKind::Eof))));
        assert_eq!(uint32(ByteOrder::LittleEndian)(&b"\x01\x02"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01\x02"[..], nom::error::ErrorKind::Eof))));
        assert_eq!(uint32(ByteOrder::LittleEndian)(&b"\x01\x02\x03"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01\x02\x03"[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_sint32_be() {
        assert_eq!(sint32(ByteOrder::BigEndian)(&b"\x00\x00\x00\x00\x10\x12"[..]), Ok((&b"\x10\x12"[..], 0 as i32)));
        assert_eq!(sint32(ByteOrder::BigEndian)(&b"\x00\x00\x00\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 4 as i32)));
        assert_eq!(sint32(ByteOrder::BigEndian)(&b"\x80\x04\x00\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], -2147221500 as i32)));
        assert_eq!(sint32(ByteOrder::BigEndian)(&b"\x00\x00\x10\xf4\x10\x12"[..]), Ok((&b"\x10\x12"[..], 4340 as i32)));
        assert_eq!(sint32(ByteOrder::BigEndian)(&b"\xff\x01\x89\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], -16676604 as i32)));

        assert_eq!(sint32(ByteOrder::BigEndian)(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
        assert_eq!(sint32(ByteOrder::BigEndian)(&b"\x01"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01"[..], nom::error::ErrorKind::Eof))));
        assert_eq!(sint32(ByteOrder::BigEndian)(&b"\x01\x02"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01\x02"[..], nom::error::ErrorKind::Eof))));
        assert_eq!(sint32(ByteOrder::BigEndian)(&b"\x01\x02\x03"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01\x02\x03"[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_sint32_le() {
        assert_eq!(sint32(ByteOrder::LittleEndian)(&b"\x00\x00\x00\x00\x10\x12"[..]), Ok((&b"\x10\x12"[..], 0 as i32)));
        assert_eq!(sint32(ByteOrder::LittleEndian)(&b"\x00\x00\x00\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 1 << 26 as i32)));
        assert_eq!(sint32(ByteOrder::LittleEndian)(&b"\x80\x04\x00\x04\x10\x12"[..]), Ok((&b"\x10\x12"[..], 67110016 as i32)));
        assert_eq!(sint32(ByteOrder::LittleEndian)(&b"\x00\x00\x10\xf4\x10\x12"[..]), Ok((&b"\x10\x12"[..], -200278016 as i32)));
        assert_eq!(sint32(ByteOrder::LittleEndian)(&b"\xff\x01\x89\x84\x10\x12"[..]), Ok((&b"\x10\x12"[..], -2071395841 as i32)));

        assert_eq!(sint32(ByteOrder::LittleEndian)(&b""[..]),  Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Eof))));
        assert_eq!(sint32(ByteOrder::LittleEndian)(&b"\x01"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01"[..], nom::error::ErrorKind::Eof))));
        assert_eq!(sint32(ByteOrder::LittleEndian)(&b"\x01\x02"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01\x02"[..], nom::error::ErrorKind::Eof))));
        assert_eq!(sint32(ByteOrder::LittleEndian)(&b"\x01\x02\x03"[..]),  Err(nom::Err::Error(nom::error::Error::new(&b"\x01\x02\x03"[..], nom::error::ErrorKind::Eof))));
    }

    #[test]
    fn test_string_utf8_be() {
        let data = b"\x00\x00\x00\x0eHello, world!\x00\x11";
        let decoder = string(ByteOrder::BigEndian, StringEncoding::Utf8,
                                     LengthSize::Length4, u32::MAX as usize);
        assert_eq!(decoder(&data[..]), Ok((&b"\x11"[..], "Hello, world!".to_string())));

    }

}