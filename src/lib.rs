#[macro_use]
extern crate nom;

use nom::{IResult, le_i32, le_i64};
use std::marker::{Sized};
use std::convert::{From};

trait Kind<A: SafeCopy> {
    fn safe_parse(i: &[u8]) -> IResult<&[u8], A>;
}

pub struct Primitive;
impl<A: SafeCopy> Kind<A> for Primitive {
    named!(safe_parse<A>, call!(A::parse_unsafe));
}

pub struct Base;
impl<A: SafeCopy> Kind<A> for Base {
    named!(safe_parse<A>, do_parse!(
        v: le_i32 >>
        res: cond_reduce!(v==A::VERSION, A::parse_unsafe) >>
        (res)
    ));
}

pub struct Extended<B>(::std::marker::PhantomData<B>);
impl<A: SafeCopy+From<B>, B: SafeCopy> Kind<A> for Extended<B> {
    named!(safe_parse<A>, alt!(
        do_parse!(
            v: le_i32 >>
            res: cond_reduce!(v==A::VERSION, A::parse_unsafe) >>
            (res)
        ) |
        call!(A::K::safe_parse)
    ));
}

trait SafeCopy: Sized {
    type K: Kind<Self>;
    fn parse_unsafe(i: &[u8]) -> IResult<&[u8], Self>;
    const VERSION: i32;
}

impl SafeCopy for i32 {
    type K = Primitive;
    named!(parse_unsafe<i32>, call!(le_i32));
    const VERSION: i32 = 0;
}

impl SafeCopy for i64 {
    type K = Primitive;
    named!(parse_unsafe<i64>, call!(le_i64));
    const VERSION: i32 = 0;
}



struct Foo(i32);

impl SafeCopy for Foo {
    type K = Base;
    named!(parse_unsafe<Foo>, do_parse!(
        val: le_i32 >>
        (Foo(val))
    ));
    const VERSION: i32 = 0;
}

struct Bar(i64);

impl From<Foo> for Bar {
    fn from(x: Foo) -> Bar {
        match x {
            Foo(val) => Bar(i64::from(val))
        }
    }
}

impl SafeCopy for Bar {
    type K = Extended<Foo>;
    named!(parse_unsafe<Bar>, do_parse!(
        val: le_i64 >>
        (Bar(val))
    ));
    const VERSION: i32 = 1;
}
