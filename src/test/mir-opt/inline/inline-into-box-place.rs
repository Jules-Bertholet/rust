// ignore-endian-big
// needs-unwind
// compile-flags: -Z mir-opt-level=4
// EMIT_MIR_FOR_EACH_BIT_WIDTH
#![feature(box_syntax)]
// EMIT_MIR inline_into_box_place.main.Inline.diff
fn main() {
    let _x: Box<Vec<u32>> = box Vec::new();
}
