extern crate bincode;

use bincode::*;
use std::boxed::Box;
use std::convert::From;
use std::io::{Read, Write};
use std::marker::Sized;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub trait Kind<A: SafeCopy> {
    fn safe_parse<R: Read>(reader: &mut R) -> Result<A>;
    fn safe_parse_versioned<R: Read>(v: i32, reader: &mut R) -> Result<A>;
    fn safe_write<W: Write>(writer: &mut W, value: &A) -> Result<()>;
}

pub struct Primitive;
impl<A: SafeCopy> Kind<A> for Primitive {
    fn safe_parse<R: Read>(reader: &mut R) -> Result<A> {
        A::parse_unsafe(reader)
    }

    fn safe_parse_versioned<R: Read>(_v: i32, _reader: &mut R) -> Result<A> {
        Err(Box::new(ErrorKind::Custom(String::from(
            "Migration with Primitive",
        ))))
    }

    fn safe_write<W: Write>(writer: &mut W, value: &A) -> Result<()> {
        A::write_unsafe(writer, value)
    }
}

pub struct Base;
impl<A: SafeCopy> Kind<A> for Base {
    fn safe_parse<R: Read>(reader: &mut R) -> Result<A> {
        let v: i32 = deserialize_from(reader, Infinite)?;
        Self::safe_parse_versioned(v, reader)
    }

    fn safe_parse_versioned<R: Read>(v: i32, reader: &mut R) -> Result<A> {
        if v == A::VERSION {
            A::parse_unsafe(reader)
        } else {
            Err(Box::new(ErrorKind::Custom(String::from("Wrong Version"))))
        }
    }

    fn safe_write<W: Write>(writer: &mut W, value: &A) -> Result<()> {
        serialize_into(writer, &A::VERSION, Infinite)?;
        A::write_unsafe(writer, value)
    }
}

pub struct Extended<B>(::std::marker::PhantomData<B>);
impl<A: SafeCopy + From<B>, B: SafeCopy> Kind<A> for Extended<B> {
    fn safe_parse<R: Read>(reader: &mut R) -> Result<A> {
        let v: i32 = deserialize_from(reader, Infinite)?;
        Self::safe_parse_versioned(v, reader)
    }

    fn safe_parse_versioned<R: Read>(v: i32, reader: &mut R) -> Result<A> {
        if v == A::VERSION {
            A::parse_unsafe(reader)
        } else {
            B::K::safe_parse_versioned(v, reader).map(From::from)
        }
    }

    fn safe_write<W: Write>(writer: &mut W, value: &A) -> Result<()> {
        serialize_into(writer, &A::VERSION, Infinite)?;
        A::write_unsafe(writer, value)
    }
}

pub trait SafeCopy: Sized {
    type K: Kind<Self>;
    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self>;
    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()>;
    const VERSION: i32;
}

impl SafeCopy for i32 {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        deserialize_from(reader, Infinite)
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        serialize_into(writer, value, Infinite)
    }
}

impl SafeCopy for i64 {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        deserialize_from(reader, Infinite)
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        serialize_into(writer, value, Infinite)
    }
}

pub fn safe_parse<A: SafeCopy>(reader: &mut impl Read) -> Result<A> {
    A::K::safe_parse(reader)
}

pub fn safe_write<A: SafeCopy>(writer: &mut impl Write, value: &A) -> Result<()> {
    A::K::safe_write(writer, value)
}

#[cfg(test)]
mod tests {
    use quickcheck::*;

    use super::*;

    fn serialize_deserialize<A: SafeCopy + std::fmt::Debug + PartialEq>(value: &A) -> TestResult {
        let mut buffer = Vec::new();
        safe_write(&mut buffer, value).unwrap();
        let mut cursor = std::io::Cursor::new(buffer);
        let result: Result<A> = safe_parse(&mut cursor);
        match result {
            Ok(x) => {
                if x == *value {
                    TestResult::passed()
                } else {
                    TestResult::error(format!("Expected {:?}, got {:?}", value, x))
                }
            },
            Err(e) => TestResult::error(format!("Error: {:?}", e)),
        }
    }

    #[quickcheck]
    fn i32(x: i32) -> TestResult {
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn i64(x: i64) -> TestResult {
        serialize_deserialize(&x)
    }
}
