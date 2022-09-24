#![feature(negative_impls)]

use std::marker::{Aligned, Copy};

enum TestE {
  A
}

struct MyType;

struct NotSync;
impl !Sync for NotSync {}

impl Aligned for TestE {}
//~^ ERROR E0322

impl Aligned for MyType {}
//~^ ERROR E0322

impl Aligned for (MyType, MyType) {}
//~^ ERROR E0322
//~| ERROR E0117

impl Aligned for &'static NotSync {}
//~^ ERROR E0322

impl Aligned for [MyType] {}
//~^ ERROR E0322
//~| ERROR E0117

impl Aligned for &'static [NotSync] {}
//~^ ERROR E0322
//~| ERROR E0117

impl Aligned for dyn Sync {}
//~^ ERROR E0322
//~| ERROR E0117

fn main() {
}
