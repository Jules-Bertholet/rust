extern "C" {
    static externalValue: isize;
}

fn main() {
    let boolValue = match 42 {
        externalValue => true,
        //~^ ERROR statics from `extern` blocks cannot be referenced in patterns
        _ => false,
    };
}
