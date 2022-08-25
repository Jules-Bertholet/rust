// check-pass
// aux-build: non-exhaustive-stuff.rs
#![feature(static_in_pattern)]
#![deny(unreachable_patterns)]

extern crate non_exhaustive_stuff;

#[derive(PartialEq, Eq)]
struct Whee(i32, bool);

#[derive(PartialEq, Eq)]
enum ZstEnum {
    Variant(),
}

#[derive(PartialEq, Eq)]
struct ZstStruct {}

#[derive(PartialEq, Eq)]
struct ZstStruct2(());

#[derive(PartialEq, Eq)]
enum OneValue<'a> {
    Foo((((),), ()), [&'a &'a (); 5], ZstStruct, ZstStruct2, [i32; 0]),
}

mod stuff {
    #[derive(PartialEq, Eq)]
    pub(super) struct PrivateZst(());

    pub(super) static PRIVATE_ZST: PrivateZst = PrivateZst(());
}

use non_exhaustive_stuff::{
    NonExhaustiveZstEnum, NON_EXHAUSTIVE_VARIANT_ZST_ENUM, NON_EXHAUSTIVE_ZST_ENUM,
    NON_EXHAUSTIVE_ZST_STRUCT,
};

static ZERO: i32 = 0;
static ZERO_REF: &&i32 = &&0;
static TRUE: bool = true;
static FALSE: bool = false;
static WHEE: Whee = Whee(1, false);
static UNIT: () = ();
static ZST_ENUM: ZstEnum = ZstEnum::Variant();
static ONE_VALUE: OneValue =
    OneValue::Foo((((),), ()), [&&(); 5], ZstStruct {}, ZstStruct2(()), []);

fn main() {
    let val = match 0 {
        ZERO => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match 1 {
        ZERO => true,
        _ => false,
    };
    assert_eq!(val, false);

    let val = match (0, 1) {
        (ZERO, 1) => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match (1, 1) {
        (ZERO, 1) => true,
        _ => false,
    };
    assert_eq!(val, false);

    let val = match Whee(1, false) {
        WHEE => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match Whee(1, true) {
        WHEE => true,
        _ => false,
    };
    assert_eq!(val, false);

    let val = match true {
        TRUE | FALSE => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match () {
        UNIT => true,
    };
    assert_eq!(val, true);

    let val = match ZstEnum::Variant() {
        ZST_ENUM => true,
    };
    assert_eq!(val, true);

    let val = match OneValue::Foo((((),), ()), [&&(); 5], ZstStruct {}, ZstStruct2(()), []) {
        ONE_VALUE => true,
    };
    assert_eq!(val, true);

    let val = match &[0, 1, 2, 3] {
        &[ZERO, ..] => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match &&0 {
        ZERO_REF => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match &&0 {
        &&ZERO => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match (3, ()) {
        (3, UNIT) => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match NonExhaustiveZstEnum::Variant() {
        NON_EXHAUSTIVE_ZST_ENUM => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match NON_EXHAUSTIVE_ZST_STRUCT {
        NON_EXHAUSTIVE_ZST_STRUCT => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match NON_EXHAUSTIVE_VARIANT_ZST_ENUM {
        NON_EXHAUSTIVE_VARIANT_ZST_ENUM => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match stuff::PRIVATE_ZST {
        stuff::PRIVATE_ZST => true,
        _ => false,
    };
    assert_eq!(val, true);

    let val = match &&0 {
        ZERO_REF => true,
        ZERO_REF => false,
        _ => false,
    };
    assert_eq!(val, true);
}
