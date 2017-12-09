extern crate bincode;

use bincode::*;
use std::marker::{Sized};
use std::convert::{From};
use std::io::{Read, Write};
use std::boxed::Box;

trait Kind<A: SafeCopy> {
    fn safe_parse<R: Read>(reader: &mut R) -> Result<A>;
    fn safe_parse_versioned<R: Read>(v: i32, reader: &mut R) -> Result<A>;
}

pub struct Primitive;
impl<A: SafeCopy> Kind<A> for Primitive {
    fn safe_parse<R: Read>(reader: &mut R) -> Result<A> {
        A::parse_unsafe(reader)
    }

    fn safe_parse_versioned<R: Read>(_v: i32, _reader: &mut R) -> Result<A> {
        Err(Box::new(ErrorKind::Custom(String::from("Migration with Primitive"))))
    }
}

pub struct Base;
impl<A: SafeCopy> Kind<A> for Base {
    fn safe_parse<R: Read>(reader: &mut R) -> Result<A> {
        deserialize_from(reader, Infinite)
            .and_then(|v: i32| {
                Self::safe_parse_versioned(v, reader)
            })
    }

    fn safe_parse_versioned<R: Read>(v: i32, reader: &mut R) -> Result<A> {
        if v==A::VERSION {
            A::parse_unsafe(reader)
        } else {
            Err(Box::new(ErrorKind::Custom(String::from("Wrong Version"))))
        }
    }
}

pub struct Extended<B>(::std::marker::PhantomData<B>);
impl<A: SafeCopy+From<B>, B: SafeCopy> Kind<A> for Extended<B> {
    fn safe_parse<R: Read>(reader: &mut R) -> Result<A> {
        deserialize_from(reader, Infinite)
            .and_then(|v: i32| {
                Self::safe_parse_versioned(v, reader)
            })
    }

    fn safe_parse_versioned<R: Read>(v: i32, reader: &mut R) -> Result<A> {
        if v==A::VERSION {
            A::parse_unsafe(reader)
        } else {
            B::K::safe_parse_versioned(v, reader).map(From::from)
        }
    }
}

trait SafeCopy: Sized {
    type K: Kind<Self>;
    fn parse_unsafe<R: Read>(bytes: &mut R) -> Result<Self>;
    const VERSION: i32;
}
