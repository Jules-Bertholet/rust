#![feature(core_intrinsics)]

use std::marker::Aligned;

struct Test {
    a: dyn Send
}

fn main() {
    let _ = core::intrinsics::min_align_of::<dyn Sized>();
                                          //~^ ERROR E0277
                                          //~| ERROR E0038
    let _ = core::intrinsics::min_align_of::<dyn Aligned>();
                                          //~^ ERROR E0277
                                          //~| ERROR E0038
    let _ = core::intrinsics::min_align_of::<Test>(); //~ ERROR E0277
}
