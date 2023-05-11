#![no_std]
#![feature(asm_const)]

//#[cfg(any(feature = "sv39", feature = "sv48"))]
use riscv::register::satp;
pub const KERNEL_BASE: usize = 0xffff_ffff_c000_0000;

#[cfg(any(feature = "sv39",feature = "sv48"))]
const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;

#[cfg(feature = "sv39")]
#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_SV39: [u64; 512] = [0; 512];

#[cfg(feature = "sv48")]
#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_SV48: [u64; 0x600] = [0; 0x600];

#[cfg(feature = "sv39")]
pub unsafe fn pre_mmu() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[0x102] = (0x80000 << 10) | 0xef;

    // 0xffff_ffff_c000_0000..highest, VRWX_GAD, 1G block
    BOOT_PT_SV39[0x1ff] = (0x80000 << 10) | 0xef;
}

#[cfg(feature = "sv48")]
pub unsafe fn pre_mmu() {
    let page_table_root = BOOT_PT_SV48.as_ptr() as u64;
    let root_page_id = page_table_root>>12;
    BOOT_PT_SV48[0] = ((root_page_id+1) << 10) | 0x01;
    BOOT_PT_SV48[0x1FF] = ((root_page_id+2) << 10) | 0x01;
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV48[0x202] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV48[0x502] = (0x80000 << 10) | 0xef;

    // 0xffff_ffff_c000_0000..highest, VRWX_GAD, 1G block
    BOOT_PT_SV48[0x5ff] = (0x80000 << 10) | 0xef;
}
#[cfg(feature = "sv39")]
pub unsafe fn enable_mmu() {
    let page_table_root = BOOT_PT_SV39.as_ptr() as usize;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}
#[cfg(feature = "sv48")]
pub unsafe fn enable_mmu() {
    let page_table_root = BOOT_PT_SV48.as_ptr() as usize;
    satp::set(satp::Mode::Sv48, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
    
}
#[cfg(any(feature = "sv39", feature = "sv48"))]
pub unsafe fn post_mmu() {
    core::arch::asm!("
        li      t0, {phys_virt_offset}  // fix up virtual high address
        add     sp, sp, t0
        add     ra, ra, t0
        ret     ",
        phys_virt_offset = const PHYS_VIRT_OFFSET,
    )
}