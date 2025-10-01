#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;


//TODO: The unsafe flag signals you're taking responsibility for layout safety. 
#[account(zero_copy(unsafe))]
#[repr(C)]
#[derive(Default, Debug, PartialEq)]
struct DemoState {
    /// We can still keep simple types.
    counter: u64,
    /// Unsafe mode allows enums or other non-Pod fields.
    flag: DemoFlag,
}

#[repr(u8)]
#[derive(Default, Debug, PartialEq, Copy, Clone)]
enum DemoFlag {
    #[default]
    Off,
    On,
}

//TODO: map existing account bytes straight into your struct 

#[test]
fn zero_copy_unsafe_basic_usage() {
    //TODO: It fills a raw byte buffer, then reinterprets that memory as DemoState. 
    //Purpose: The attribute is what tells Anchor "this struct's memory layout is safe
    // to treat as account bytes"
    // Pretend this buffer is Solana account data loaded into memory.

    //TODO: pretending to be onchain account bytes 
    let mut raw_data = vec![0_u8; core::mem::size_of::<DemoState>()];

    unsafe {
        // `zero_copy(unsafe)` lets us interpret the bytes as `DemoState`
        // even though it has an enum that Anchor normally forbids.
        //TODO: raw_data.as_mut_ptr() grabs the raw pointer to the first byte of the Vec<u8>.
        let state_ptr = raw_data.as_mut_ptr() as *mut DemoState;
        //TODO: stores a DemoState directly into the raw byte buffer, giving that buffer the exact layout the struct expects.
        state_ptr.write(DemoState {
            counter: 42,
            flag: DemoFlag::On,
        });

        let state_ref = &*(raw_data.as_ptr() as *const DemoState);

        assert_eq!(state_ref.counter, 42);
        assert_eq!(state_ref.flag, DemoFlag::On);
    }
}

// Implement minimal Anchor setup so the attribute compiles during tests.
declare_id!("11111111111111111111111111111111");

