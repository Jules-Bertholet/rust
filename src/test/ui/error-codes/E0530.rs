enum Enum {
    WithField(i32)
}

use Enum::*;


fn main() {
    match WithField(1) {
        WithField => {}
        //~^ ERROR E0530
    }
}
