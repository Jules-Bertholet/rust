// run-pass

#![feature(core_intrinsics)]

use std::marker::Aligned;

trait ObjectSafe {
    fn foo<T>(&self) where Self: Aligned;
}

impl ObjectSafe for [i32] {
    fn foo<T>(&self) {
        dbg!(core::intrinsics::min_align_of::<T>());
        dbg!(self);
    }
}

impl ObjectSafe for i32 {
    fn foo<T>(&self) {
        dbg!(core::intrinsics::min_align_of::<T>());
        dbg!(self);
    }
}

fn main() {
    assert_eq!(core::intrinsics::min_align_of::<[u32]>(), core::intrinsics::min_align_of::<u32>());

    let a: &[i32] = &[3];
    a.foo::<u32>();

    let _: &dyn ObjectSafe = &3;
}
