static foo: i32 = 0;

fn bar(foo: i32) {}
//~^ ERROR referencing statics in patterns is experimental

mod submod {
    pub static answer: i32 = 42;
}

use self::submod::answer;

fn question(answer: i32) {}
//~^ ERROR referencing statics in patterns is experimental
fn main() {
}
