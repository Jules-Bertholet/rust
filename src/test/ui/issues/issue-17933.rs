pub static X: usize = 1;

fn main() {
    match 1 {
        self::X => { },
        //~^ ERROR referencing statics in patterns is experimental
        _       => { },
    }
}
