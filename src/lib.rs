#[macro_use]
extern crate nom;

use nom::{IResult, le_i32};
use std::marker::{Sized};
use std::convert::{From};

trait Kind<A: SafeCopy> {
    fn safe_parse(i: &[u8]) -> IResult<&[u8], A>;
}

struct Base;
impl<A: SafeCopy> Kind<A> for Base {
    named!(safe_parse<A>, do_parse!(
        v: le_i32 >>
        res: cond_reduce!(v==A::version(), A::parse) >>
        (res)
    ));
}

struct Extended<B>(::std::marker::PhantomData<B>);
impl<A: SafeCopy+From<B>, B: SafeCopy> Kind<A> for Extended<B> {
    named!(safe_parse<A>, alt!(
        do_parse!(
            v: le_i32 >>
            res: cond_reduce!(v==A::version(), A::parse) >>
            (res)
        ) |
        call!(A::K::safe_parse)
    ));
}

trait SafeCopy: Sized {
    type K: Kind<Self>;
    fn parse(i: &[u8]) -> IResult<&[u8], Self>;
    fn version() -> i32;
}

struct Foo();

impl SafeCopy for Foo {
    type K = Base;
    named!(parse<Foo>, do_parse!(
        (Foo())
    ));
    fn version() -> i32 { 0 }
}

struct Bar(i32);

impl From<Foo> for Bar {
    fn from(_x: Foo) -> Bar {
        Bar(0)
    }
}

impl SafeCopy for Bar {
    type K = Extended<Foo>;
    named!(parse<Bar>, do_parse!(
        val: le_i32 >>
        (Bar(val))
    ));
    fn version() -> i32 { 1 }
}
