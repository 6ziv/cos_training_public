#![no_std]
#![feature(asm_const)]

#[cfg(any(feature = "enable", feature = "disable"))]
use riscv::register::satp;


pub const KERNEL_BASE: usize = 0xffff_ffff_c000_0000;
#[cfg(feature = "enable")]
const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;

#[cfg(feature = "enable")]
#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_SV39: [u64; 512] = [0; 512];

#[cfg(feature = "enable")]
pub unsafe fn pre_mmu() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[0x102] = (0x80000 << 10) | 0xef;

    // 0xffff_ffff_c000_0000..highest, VRWX_GAD, 1G block
    BOOT_PT_SV39[0x1ff] = (0x80000 << 10) | 0xef;
}

#[cfg(feature = "disable")]
pub unsafe fn pre_mmu() {}

#[cfg(feature = "enable")]
pub unsafe fn enable_mmu() {
    let page_table_root = BOOT_PT_SV39.as_ptr() as usize;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}
#[cfg(feature = "disable")]
pub unsafe fn enable_mmu() {
    //let page_table_root = BOOT_PT_SV39.as_ptr() as usize;
    //satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    satp::set(satp::Mode::Bare,0,0);
    riscv::asm::sfence_vma_all();
}

#[cfg(feature = "enable")]
pub unsafe fn post_mmu() {
    core::arch::asm!("
        li      t0, {phys_virt_offset}  // fix up virtual high address
        add     sp, sp, t0
        add     ra, ra, t0
        ret     ",
        phys_virt_offset = const PHYS_VIRT_OFFSET,
    )
}
#[cfg(feature = "disable")]
pub unsafe fn post_mmu() {}
