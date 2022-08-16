static ZERO: i32 = 0;

pub fn main() {
    match 0 {
        ZERO => true,
        //~^ ERROR referencing statics in patterns is experimental
        _ => false,
    };
}
