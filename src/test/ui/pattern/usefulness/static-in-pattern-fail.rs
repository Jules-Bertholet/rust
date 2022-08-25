#![feature(static_in_pattern)]
#![feature(thread_local)]

struct NoPartialEqEq(i32, bool);

static mut ZERO: i32 = 0;
static ZERO_REF: &&f32 = &&0.0;
#[thread_local]
static TRUE: bool = true;
static FALSE: bool = false;
static WHEE: NoPartialEqEq = NoPartialEqEq(1, false);
#[link_name="IN"]
static OUT: i32 = 3;
extern "Rust" {
    static IN: i32;
}
static MAIN: fn() = crate::main;

fn main() {
     match 0 {
        ZERO => true,
        //~^ ERROR mutable statics cannot be referenced in patterns
        _ => false,
    };

     match (0, 1) {
        (ZERO, 1) => true,
         //~^ ERROR mutable statics cannot be referenced in patterns
        _ => false,
    };

     match 0 {
        IN => true,
        //~^ ERROR statics from `extern` blocks cannot be referenced in patterns
        _ => false,
    };

     match NoPartialEqEq(1, true) {
        WHEE => true,
        //~^ ERROR to use a constant or static of type `NoPartialEqEq` in a pattern, `NoPartialEqEq` must be annotated with `#[derive(PartialEq, Eq)]`
        _ => false,
    };

     match true {
        TRUE | FALSE => true,
        //~^ ERROR `#[thread_local]` statics cannot be referenced in patterns
        _ => false,
    };

     match crate::main as fn() {
        MAIN => true,
        //~^ ERROR function pointers cannot be used in patterns
        _ => false,
    };
}
