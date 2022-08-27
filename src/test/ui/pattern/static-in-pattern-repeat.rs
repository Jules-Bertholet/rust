#![feature(static_in_pattern)]
#![deny(unreachable_patterns)]

static ZERO: i32 = 0;
static ZERO_REF: &i32 = &0;


fn main() {
    let val = match 0 {
        ZERO => true,
        ZERO => true,
        //~^ ERROR unreachable pattern
        _ => false,
    };

    let val = match &0 {
        ZERO_REF => true,
        ZERO_REF => false,
        //~^ ERROR unreachable pattern
        _ => false,
    };
    assert_eq!(val, true);
}
