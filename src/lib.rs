extern crate bincode;

use bincode::*;
use std::borrow::Cow;
use std::boxed::Box;
use std::convert::{From, TryInto};
use std::io::{Read, Write};
use std::marker::Sized;
use std::rc::Rc;
use std::sync::Arc;

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

pub fn safe_parse<A: SafeCopy>(reader: &mut impl Read) -> Result<A> {
    A::K::safe_parse(reader)
}

pub fn safe_write<A: SafeCopy>(writer: &mut impl Write, value: &A) -> Result<()> {
    A::K::safe_write(writer, value)
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

impl SafeCopy for String {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        let len: usize = deserialize_from(reader, Infinite)?;
        let mut buf = Vec::with_capacity(len);
        reader.take(len as u64).read_to_end(&mut buf)?;
        Ok(String::from_utf8(buf)
            // FIXME: Enable more error types
            .unwrap())
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        let bytes = value.as_bytes();
        serialize_into(writer, &bytes.len(), Infinite)?;
        writer.write_all(bytes)?;
        Ok(())
    }
}

impl<T: SafeCopy> SafeCopy for Option<T> {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        let tag: u8 = deserialize_from(reader, Infinite)?;
        match tag {
            0 => Ok(None),
            1 => Ok(Some(safe_parse(reader)?)),
            _ => Err(Box::new(ErrorKind::Custom(String::from("Wrong tag")))),
        }
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        match value {
            None => serialize_into(writer, &0u8, Infinite)?,
            Some(x) => {
                serialize_into(writer, &1u8, Infinite)?;
                safe_write(writer, x)?;
            }
        }
        Ok(())
    }
}

impl<T: SafeCopy> SafeCopy for Vec<T> {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        let len: u64 = deserialize_from(reader, Infinite)?;
        let mut result = Vec::with_capacity(len.try_into().unwrap());
        for _ in 0..len {
            result.push(safe_parse(reader)?);
        }
        Ok(result)
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        let len: u64 = value.len().try_into().unwrap();
        serialize_into(writer, &len, Infinite)?;
        for x in value {
            safe_write(writer, x)?;
        }
        Ok(())
    }
}

impl<T: SafeCopy> SafeCopy for Box<T> {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Box::new(safe_parse(reader)?))
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        safe_write(writer, &**value)
    }
}

impl<T: SafeCopy> SafeCopy for Rc<T> {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Rc::new(safe_parse(reader)?))
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        safe_write(writer, &**value)
    }
}

impl<T: SafeCopy> SafeCopy for Arc<T> {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Arc::new(safe_parse(reader)?))
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        safe_write(writer, &**value)
    }
}

impl<T: SafeCopy + Clone> SafeCopy for Cow<'_, T> {
    type K = Primitive;
    const VERSION: i32 = 0;

    fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Cow::Owned(safe_parse(reader)?))
    }

    fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
        safe_write(writer, &**value)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

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
            }
            Err(e) => TestResult::error(format!("Error: {:?}", e)),
        }
    }

    #[quickcheck]
    fn prop_i32(x: i32) -> TestResult {
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_i64(x: i64) -> TestResult {
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_string(x: String) -> TestResult {
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_option(x: Option<i32>) -> TestResult {
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_vec(x: Vec<i32>) -> TestResult {
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_box(x: Box<i32>) -> TestResult {
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_rc(x_raw: i32) -> TestResult {
        let x = Rc::new(x_raw);
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_arc(x: Arc<i32>) -> TestResult {
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_cow_owned(x_raw: i32) -> TestResult {
        let x: Cow<'_, i32> = Cow::Owned(x_raw);
        serialize_deserialize(&x)
    }

    #[quickcheck]
    fn prop_cow_borrowed(x_raw: i32) -> TestResult {
        let x = Cow::Borrowed(&x_raw);
        serialize_deserialize(&x)
    }

    #[derive(Debug, PartialEq)]
    struct T1(i32);
    impl SafeCopy for T1 {
        type K = Base;
        const VERSION: i32 = 0;
        fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
            Ok(T1(safe_parse(reader)?))
        }
        fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
            safe_write(writer, &value.0)
        }
    }

    #[derive(Debug, PartialEq)]
    struct T2(i32, Option<String>);
    impl SafeCopy for T2 {
        type K = Extended<T1>;
        const VERSION: i32 = 1;
        fn parse_unsafe<R: Read>(reader: &mut R) -> Result<Self> {
            Ok(T2(safe_parse(reader)?, safe_parse(reader)?))
        }
        fn write_unsafe<W: Write>(writer: &mut W, value: &Self) -> Result<()> {
            safe_write(writer, &value.0)?;
            safe_write(writer, &value.1)
        }
    }

    impl From<T1> for T2 {
        fn from(t: T1) -> Self {
            T2(t.0, None)
        }
    }

    #[quickcheck]
    fn prop_custom(i: i32, s: Option<String>) -> TestResult {
        let t1 = T1(i);
        let mut buffer = Vec::new();
        safe_write(&mut buffer, &t1).unwrap();
        let mut cursor = std::io::Cursor::new(buffer);
        let t2: Result<T2> = safe_parse(&mut cursor);
        match t2 {
            Ok(t2) => {
                if t2 != T2(i, None) {
                    return TestResult::error(format!("Expected {:?}, got {:?}", T2(i, None), t2));
                }
            }
            Err(e) => return TestResult::error(format!("Error: {:?}", e)),
        }
        let t2 = T2(i, s);
        serialize_deserialize(&t2)
    }
}
