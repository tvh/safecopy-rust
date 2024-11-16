use quickcheck::TestResult;
use safecopy_macros::*;
use safecopy::*;

extern crate quickcheck;
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

fn serialize_deserialize<A: SafeCopy + std::fmt::Debug + PartialEq>(value: &A) -> TestResult {
    let mut buffer = Vec::new();
    safe_write(&mut buffer, value).unwrap();
    let mut cursor = std::io::Cursor::new(buffer);
    let result: Result<A, _> = safe_parse(&mut cursor);
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
fn test_unit() -> TestResult {
    #[derive(SafeCopy, Debug, PartialEq)]
    struct Test;

    serialize_deserialize(&Test)
}

#[quickcheck]
fn test_named_single_value(x: i32) -> TestResult {
    #[derive(SafeCopy, Debug, PartialEq)]
    struct Test {
        a: i32,
    }

    serialize_deserialize(&Test { a: x })
}

#[quickcheck]
fn test_unnamed_single_value(x: i32) -> TestResult {
    #[derive(SafeCopy, Debug, PartialEq)]
    struct Test(i32);

    serialize_deserialize(&Test(x))
}

#[quickcheck]
fn test_enum_without_value(x: i32) -> TestResult {
    #[derive(SafeCopy, Debug, PartialEq)]
    enum Test {
        A,
        B,
        C,
    }

    let test = match x % 3 {
        0 => Test::A,
        1 => Test::B,
        2 => Test::C,
        _ => panic!("Invalid value"),
    };

    serialize_deserialize(&test)
}

#[quickcheck]
fn test_enum_with_value(x: i32, variant: u8) -> TestResult {
    #[derive(SafeCopy, Debug, PartialEq)]
    enum Test {
        A(i32),
        B(i32),
        C(i32),
    }

    let test = match variant % 3 {
        0 => Test::A(x),
        1 => Test::B(x),
        2 => Test::C(x),
        _ => panic!("Invalid value"),
    };

    serialize_deserialize(&test)
}