#![no_std]
#![no_main]

use drv0 as _;
use drv1 as _;

use drv_common::CallEntry;
use core::{slice,mem};
extern "C"{
    fn initcalls_start()->u64;
    fn initcalls_end()->u64;
}
#[no_mangle]
fn main() {
    libos::init();

    libos::println!("\n[ArceOS Tutorial]: B0\n");
    verify();
}

/* Todo: Implement it */
fn traverse_drivers() {
    //libos::println!("\n!!! Fix it !!!\n");
    // Parse range of init_calls by calling C function.
    let (range_start,range_end);
    unsafe{
        range_start = initcalls_start() as usize;
        range_end   = initcalls_end()   as usize;
    }
    display_initcalls_range(range_start, range_end);
    let call_entries = unsafe{
        slice::from_raw_parts::<'static,CallEntry>(range_start as *const CallEntry, (range_end-range_start) / mem::size_of::<CallEntry>())
    };
    call_entries.into_iter().for_each(
        |entry| {
            let driver = (entry.init_fn)();
            display_drv_info(driver.name,driver.compatible);
        }
    )
    // For each driver, display name & compatible
    // display_drv_info(drv.name, drv.compatible);
}

fn display_initcalls_range(start: usize, end: usize) {
    libos::println!("init calls range: 0x{:X} ~ 0x{:X}\n", start, end);
}

fn display_drv_info(name: &str, compatible: &str) {
    libos::println!("Found driver '{}': compatible '{}'", name, compatible);
}

fn verify() {
    traverse_drivers();

    libos::println!("\nResult: Okay!");
}
