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
fn test_single_value(x: i32) -> TestResult {
    #[derive(SafeCopy, Debug, PartialEq)]
    struct Test {
        a: i32,
    }

    serialize_deserialize(&Test { a: x })
}
