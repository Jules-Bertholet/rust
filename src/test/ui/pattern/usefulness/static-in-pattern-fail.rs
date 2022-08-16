#![feature(static_in_pattern)]
#![feature(thread_local)]

struct NoPartialEqEq(i32, bool);

static mut ZERO: i32 = 0;
static ZERO_REF: &&f32 = &&0.0;
#[thread_local]
static TRUE: bool = true;
static FALSE: bool = false;
static WHEE: NoPartialEqEq = NoPartialEqEq(1, false);

fn main() {
    let val = match 0 {
        ZERO => true,
        //~^ ERROR mutable statics cannot be referenced in patterns
        _ => false,
    };

    let val = match (0, 1) {
        (ZERO, 1) => true,
         //~^ ERROR mutable statics cannot be referenced in patterns
        _ => false,
    };

    let val = match NoPartialEqEq(1, true) {
        WHEE => true,
        //~^ ERROR to use a constant or static of type `NoPartialEqEq` in a pattern, `NoPartialEqEq` must be annotated with `#[derive(PartialEq, Eq)]`
        _ => false,
    };

    let val = match true {
        TRUE | FALSE => true,
        //~^ ERROR `#[thread_local]` statics cannot be referenced in patterns
        _ => false,
    };
}
