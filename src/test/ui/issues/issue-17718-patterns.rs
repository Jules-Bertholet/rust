static A1: usize = 1;
static mut A2: usize = 1;
const A3: usize = 1;

fn main() {
    match 1 {
        A1 => {} //~ ERROR: referencing statics in patterns is experimental
        A2 => {} //~ ERROR: mutable statics cannot be referenced in patterns
        A3 => {}
        _ => {}
    }
}
