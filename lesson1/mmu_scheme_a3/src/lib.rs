#![no_std]
#![feature(asm_const)]

//#[cfg(any(feature = "sv39", feature = "sv48"))]
use riscv::register::satp;
pub const KERNEL_BASE: usize = 0xffff_ffff_c000_0000;

#[cfg(any(feature = "sv39",feature = "sv48"))]
const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;

const PAGE_TABLE_BUFFER_SIZE:usize = 256;

#[cfg(any(feature = "sv39",feature = "sv48"))]
#[link_section = ".data.boot_page_table"]
static mut BOOT_PT: [[u64; 512];PAGE_TABLE_BUFFER_SIZE] = [[0; 512];PAGE_TABLE_BUFFER_SIZE];
static mut BOOT_PT_CNT:usize = 0;

#[cfg(feature = "sv39")]
const MAX_DEPTH:usize = 3;
#[cfg(feature = "sv48")]
const MAX_DEPTH:usize = 4;


unsafe fn map_page_impl(ptid:usize, vpn:[usize;MAX_DEPTH], pte:u64, lv: usize, cur_lv: usize){
    if cur_lv == lv{
        if (BOOT_PT[ptid][vpn[cur_lv]]&1)!=0 {
            panic!("mapping conflicts!");
        }else{
            BOOT_PT[ptid][vpn[cur_lv]] = pte;
        }
    }else if cur_lv > lv{
        let next_pt_index:usize = 
        if (BOOT_PT[ptid][vpn[cur_lv]]&1)!=0 {
            (BOOT_PT[ptid][vpn[cur_lv]] >> 10) as usize - ((BOOT_PT.as_ptr() as usize)>>12)
        }else{
            let r = BOOT_PT_CNT;
            let page_table_root = BOOT_PT.as_ptr() as u64;
            let root_page_id = page_table_root>>12;
            BOOT_PT[ptid][vpn[cur_lv]] = ((root_page_id + (r as u64)) << 10) | 1;
            BOOT_PT_CNT+=1;
            r
        };
        map_page_impl(next_pt_index,vpn,pte,lv,cur_lv-1);
    }else{
        panic!("Bad level");
    }
}

enum PageSize{
    SimplePage,
    MegaPage,
    GigaPage,
    TeraPage,
}
impl PageSize{
    const   _4K:PageSize = PageSize::SimplePage;
    const   _2M:PageSize = PageSize::MegaPage;
    const   _1G:PageSize = PageSize::GigaPage;
    const _512G:PageSize = PageSize::TeraPage;
}
fn va2vpn(va64:u64)->[usize;MAX_DEPTH]{
    let va = va64 as usize;
    let mut ret:[usize;MAX_DEPTH]=[0usize;MAX_DEPTH];
    let slice = &([(va>>12)&0x1FF, (va>>21)&0x1FF, (va>>30)&0x1FF,(va>>39)&0x1FF])[0..MAX_DEPTH];
    ret.copy_from_slice(slice);
    ret
}
fn ps2lv(ps:PageSize)->usize{
    match ps {
        PageSize::SimplePage => 0,
        PageSize::MegaPage   => 1,
        PageSize::GigaPage   => 2,
        PageSize::TeraPage   => 3,
    }
}
unsafe fn map_page(va: u64, pa: u64, ps:PageSize,flags: u64){
    let pte = ((pa>>2) & (!0x3FF)) | (flags&0x3FF);
    map_page_impl(0,va2vpn(va),pte,ps2lv(ps),MAX_DEPTH-1);
    //map_page_impl()
}

macro_rules! map_pages {
    ( $( ($va:expr, $pa:expr, $ps:expr) ),* ) => {
        {
            BOOT_PT_CNT = 1;
            $(
                map_page($va,$pa,$ps,0xef);
            )*
        }
    };
}

pub unsafe fn pre_mmu() {
    map_pages![
        (0x8000_0000,0x8000_0000,PageSize::_1G),
        (0xffff_ffc0_8000_0000,0x8000_0000,PageSize::_1G),
        (0xffff_ffff_c000_0000,0x8000_0000,PageSize::_1G)
    ]
}

#[cfg(feature = "sv39")]
pub unsafe fn enable_mmu() {
    let page_table_root = BOOT_PT.as_ptr() as usize;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}
#[cfg(feature = "sv48")]
pub unsafe fn enable_mmu() {
    let page_table_root = BOOT_PT.as_ptr() as usize;
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