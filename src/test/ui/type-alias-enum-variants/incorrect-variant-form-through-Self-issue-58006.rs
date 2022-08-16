pub enum Enum {
    A(usize),
}

impl Enum {
    fn foo(&self) -> () {
        match self {
            Self::A => (),
            //~^ ERROR expected unit struct, unit variant, constant, or static, found tuple variant
        }
    }
}

fn main() {}
