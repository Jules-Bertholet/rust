#![feature(static_in_pattern)]
#![deny(unreachable_patterns)]

static ZERO: i32 = 0;

fn main() {
    let val = match 0 {
        ZERO => true,
        ZERO => true,
        //~^ ERROR unreachable pattern
        _ => false,
    };
}
