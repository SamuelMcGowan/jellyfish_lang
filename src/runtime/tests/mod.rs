use std::mem::size_of;

use super::chunk::Opcode;
use super::value::Value;

#[test]
fn value_is_small() {
    println!("Value size: {:?}", size_of::<Value>());

    // this isn't that small but `u64`s are already 8 bytes
    // and I don't want to come up with something cleverer
    assert!(size_of::<Value>() <= 16);
}

#[test]
fn opcode_is_small() {
    println!("Opcode size: {:?}", size_of::<Opcode>());

    assert!(size_of::<Opcode>() == 1)
}
