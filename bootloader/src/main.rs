// Most of this code is taken from the phobos project.

#![no_std]
#![no_main]
#![feature(abi_efiapi)]

extern crate alloc;

use arrayvec::ArrayVec;

use log::info;
use uefi::{
    prelude::*,
    table::boot::{AllocateType, MemoryType},
};

use x86_64::{
    align_up,
    registers::control::{Cr0, Cr0Flags, Cr3, Cr4, Efer},
    structures::paging::{FrameAllocator, PageTable, PageTableFlags, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

use alloc::{borrow::ToOwned, format, vec, vec::Vec};
use boot_lib::{
    KernelArgs, KERNEL_ARGS_MEM_TYPE, KERNEL_STACK_BOTTOM, KERNEL_STACK_MEM_TYPE,
    KERNEL_STACK_SIZE_PAGES, PHYS_MAP_OFFSET, PTE_MEM_TYPE,
};
use core::{
    arch::asm,
    iter::FromIterator,
    mem::{size_of, MaybeUninit},
    ptr::addr_of_mut,
};
use uefi::proto::console::gop::{FrameBuffer, GraphicsOutput, ModeInfo, PixelFormat::Bgr};
use x86_64::structures::paging::{
    mapper::{MapToError, TranslateResult},
    Mapper, OffsetPageTable, Page, PageSize, Size1GiB, Size2MiB, Translate,
};

use kernel::kernel_main;

struct UefiAlloc;

unsafe impl FrameAllocator<Size4KiB> for UefiAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let addr = unsafe { uefi_services::system_table().as_mut() }
            .boot_services()
            .allocate_pages(AllocateType::AnyPages, MemoryType::custom(PTE_MEM_TYPE), 1)
            .expect("Failed to allocate a page");
        Some(PhysFrame::from_start_address(PhysAddr::new(addr)).unwrap())
    }
}

unsafe fn map_sized<S: PageSize, A: FrameAllocator<Size4KiB>, M>(
    virt: VirtAddr,
    phys: PhysAddr,
    pages: u64,
    flags: PageTableFlags,
    parent_flags: PageTableFlags,
    map: &mut M,
    alloc: &mut A,
) -> u64
where
    M: Mapper<S> + Sized,
{
    let mut left = pages;
    let small_pages = S::SIZE / Size4KiB::SIZE;
    if virt.is_aligned(S::SIZE) && phys.is_aligned(S::SIZE) {
        while left >= small_pages {
            let offset = (pages - left) * Size4KiB::SIZE;
            map.map_to_with_table_flags(
                Page::<S>::from_start_address(virt + offset).unwrap(),
                PhysFrame::from_start_address(phys + offset).unwrap(),
                flags,
                parent_flags,
                alloc,
            )
            .map_err(|e| match e {
                MapToError::FrameAllocationFailed => "MapToError::FrameAllocationFailed".to_owned(),
                MapToError::ParentEntryHugePage => "MapToError::ParentEntryHugePage".to_owned(),
                MapToError::PageAlreadyMapped(x) => {
                    format!("MapToError::PageAlreadyMapped({:?})", x)
                }
            })
            .expect("Mapping failed")
            .flush();
            left -= small_pages;
        }
        pages - left
    } else {
        0
    }
}

/// Map a large amount of memory
unsafe fn map_offset<A: FrameAllocator<Size4KiB>>(
    virt: VirtAddr,
    phys: PhysAddr,
    pages: u64,
    map: &mut OffsetPageTable,
    alloc: &mut A,
    flags: PageTableFlags,
    parent_flags: PageTableFlags,
) {
    assert!(virt.is_aligned(Size4KiB::SIZE));
    let mut done = 0;
    if pages
        .checked_mul(Size4KiB::SIZE)
        .and_then(|x| virt.as_u64().checked_add(x))
        .is_none()
    {
        panic!("Not enough memory to create mapping")
    }

    info!(
        "Mapping {:?} - {:?} --> {:?} - {:?}",
        virt,
        virt + Size4KiB::SIZE * pages,
        PhysAddr::new(0),
        PhysAddr::new(Size4KiB::SIZE * pages)
    );

    done += map_sized::<Size1GiB, A, _>(
        virt + done * Size4KiB::SIZE,
        phys + done * Size4KiB::SIZE,
        pages - done,
        flags,
        parent_flags,
        map,
        alloc,
    );

    done += map_sized::<Size2MiB, A, _>(
        virt + done * Size4KiB::SIZE,
        phys + done * Size4KiB::SIZE,
        pages - done,
        flags,
        parent_flags,
        map,
        alloc,
    );

    done += map_sized::<Size4KiB, A, _>(
        virt + done * Size4KiB::SIZE,
        phys + done * Size4KiB::SIZE,
        pages - done,
        flags,
        parent_flags,
        map,
        alloc,
    );

    assert_eq!(done, pages);
}

unsafe fn map_stack<M>(
    bottom: VirtAddr,
    size_pages: u64,
    st: &mut SystemTable<Boot>,
    mapper: &mut M,
) where
    M: Mapper<Size4KiB>,
{
    assert!(size_pages > 0);
    let mem = st
        .boot_services()
        .allocate_pages(
            AllocateType::AnyPages,
            MemoryType::custom(KERNEL_STACK_MEM_TYPE),
            size_pages as _,
        )
        .expect("Could not allocate memory for the kernel stack");

    for i in 0..size_pages {
        mapper
            .map_to_with_table_flags(
                Page::containing_address(bottom - Size4KiB::SIZE * i),
                PhysFrame::containing_address(PhysAddr::new(
                    mem + Size4KiB::SIZE * (size_pages - i),
                )),
                PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE | PageTableFlags::WRITABLE,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                &mut UefiAlloc {},
            )
            .expect("Failed to map kernel stack")
            .flush();
    }
}

fn init_framebuffer(system_table: &mut SystemTable<Boot>) -> (FrameBuffer<'static>, ModeInfo) {
    let gop = unsafe {
        system_table
            .boot_services()
            .locate_protocol::<GraphicsOutput>()
            .expect("Failed to locate graphics output protocol")
            .get()
            .as_mut()
            .unwrap()
    };

    let mode = [(1920, 1080), (1920, 1200), (1280, 720), (640, 480)]
        .into_iter()
        .find_map(|i| {
            for mode in gop.modes().collect::<Vec<_>>() {
                let info = mode.info();
                if info.resolution() == i && info.pixel_format() == Bgr {
                    match gop.set_mode(&mode) {
                        Ok(_) => return Some(*mode.info()),
                        Err(_) => return None,
                    }
                }
            }
            None
        })
        .expect("Failed to find a suitable framebuffer mode");

    (gop.frame_buffer(), mode)
}

#[entry]
fn efi_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    x86_64::instructions::interrupts::disable();
    uefi_services::init(&mut system_table).expect("Failed to setup UEFI services");
    system_table
        .stdout()
        .reset(false)
        .expect("Failed to reset stdout");

    info!("Tyto x86_64 UEFI bootloader v{}", env!("CARGO_PKG_VERSION"));
    let rev = system_table.uefi_revision();
    info!("UEFI v{}.{}", rev.major(), rev.minor());
    info!("CR0 -> {:?}", Cr0::read());
    info!("CR4 -> {:?}", Cr4::read());
    info!("EFER -> {:?}", Efer::read());
    let (pml4_frame, cr3_flags) = Cr3::read();
    info!("PML4 -> {:#x}", pml4_frame.start_address().as_u64());

    info!("Initializing framebuffer");

    let (mut framebuffer, framebuffer_mode) = init_framebuffer(&mut system_table);

    info!("Loading memory map");

    let mmap_size = system_table.boot_services().memory_map_size().map_size + 0x2000;
    let mut mmap_buf = vec![0; mmap_size];

    let (_, mmap_it) = system_table
        .boot_services()
        .memory_map(&mut mmap_buf)
        .expect("Failed to get memory map");

    let mmap = ArrayVec::<_, 512>::from_iter(mmap_it.map(Clone::clone));

    info!("Mapping physical memory at offset {:#x}", PHYS_MAP_OFFSET);

    let new_pml4 = system_table
        .boot_services()
        .allocate_pages(AllocateType::AnyPages, MemoryType::custom(PTE_MEM_TYPE), 1)
        .expect("Failed to allocate new PML4");

    unsafe {
        (new_pml4 as *mut u8).copy_from(
            pml4_frame.start_address().as_u64() as *mut u8,
            Size4KiB::SIZE as _,
        );
        Cr3::write(
            PhysFrame::from_start_address(PhysAddr::new(new_pml4)).unwrap(),
            cr3_flags,
        );
    }

    let cr0 = Cr0::read();
    unsafe { Cr0::write(cr0 & !Cr0Flags::WRITE_PROTECT) };

    let mut page_table = unsafe {
        OffsetPageTable::new(
            &mut *(Cr3::read().0.start_address().as_u64() as *mut PageTable),
            VirtAddr::new(0),
        )
    };

    unsafe {
        map_offset(
            VirtAddr::new(PHYS_MAP_OFFSET as _),
            PhysAddr::new(0),
            mmap.last()
                .map(|d| d.phys_start / Size4KiB::SIZE + d.page_count)
                .unwrap(),
            &mut page_table,
            &mut UefiAlloc {},
            PageTableFlags::empty()
                | PageTableFlags::GLOBAL
                | PageTableFlags::WRITABLE
                | PageTableFlags::PRESENT
                | PageTableFlags::NO_EXECUTE,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )
    }

    unsafe { Cr0::write(cr0) };

    let mut page_table = unsafe {
        OffsetPageTable::new(
            &mut *(Cr3::read().0.start_address().as_u64() as *mut PageTable),
            VirtAddr::new(PHYS_MAP_OFFSET as _),
        )
    };

    info!("Setting virtual address map");

    let (_, mmap_it) = system_table
        .boot_services()
        .memory_map(&mut mmap_buf)
        .expect("Failed to get memory map");

    let mut mmap = ArrayVec::<_, 512>::from_iter(mmap_it.map(Clone::clone));

    info!("Loading kernel");

    info!(
        "Allocating kernel stack and mapping it at {:#x}",
        KERNEL_STACK_BOTTOM
    );

    unsafe {
        map_stack(
            VirtAddr::new(KERNEL_STACK_BOTTOM),
            KERNEL_STACK_SIZE_PAGES,
            &mut system_table,
            &mut page_table,
        );
    }

    info!("Initializing kernel args struct");

    let args = unsafe {
        &mut *((system_table
            .boot_services()
            .allocate_pages(
                AllocateType::AnyPages,
                MemoryType::custom(KERNEL_ARGS_MEM_TYPE),
                (align_up(size_of::<KernelArgs>() as u64, Size4KiB::SIZE) / Size4KiB::SIZE)
                    as usize,
            )
            .expect("Could not allocate kernel args")
            + PHYS_MAP_OFFSET) as usize as *mut MaybeUninit<KernelArgs>)
    };

    let args_ptr = args.as_mut_ptr();

    match page_table.translate(VirtAddr::new(kernel_main as usize as u64)) {
        TranslateResult::Mapped { flags, .. } => unsafe {
            if flags.contains(PageTableFlags::NO_EXECUTE) {
                panic!("Kernel entry point non-executable {:?}", flags)
            }

            info!("Exiting boot services and calling kernel entry point");

            let (mut uefi_rst, _) = system_table
                .exit_boot_services(handle, &mut mmap_buf)
                .expect("Failed to exit UEFI boot services");

            mmap.iter_mut()
                .for_each(|x| x.virt_start = x.phys_start + PHYS_MAP_OFFSET);

            let current_system_table_addr = uefi_rst.get_current_system_table_addr();

            uefi_rst = uefi_rst
                .set_virtual_address_map(
                    mmap.as_mut_slice(),
                    current_system_table_addr + PHYS_MAP_OFFSET,
                )
                .expect("Setting UEFI memory map failed");

            addr_of_mut!((*args_ptr).uefi_rst).write(uefi_rst);
            addr_of_mut!((*args_ptr).mmap).write(mmap);
            addr_of_mut!((*args_ptr).framebuffer_addr)
                .write((framebuffer.as_mut_ptr() as u64 + PHYS_MAP_OFFSET) as _);
            addr_of_mut!((*args_ptr).framebuffer_info).write(framebuffer_mode);

            let args_ptr = args.assume_init_mut() as *mut KernelArgs;

            execute_kernel(kernel_main, args_ptr);
        },
        e => panic!("Kernel entry point inaccessible: {:?}", e),
    }
}

fn execute_kernel(
    // to ensure that the types are correct.
    _kernel_main: fn(KernelArgs) -> !,
    kernel_args: *mut KernelArgs,
) -> ! {
    // Switch the stack and call the entry point according to Microsoft x64
    // calling convention

    unsafe {
        asm!(
            "mov rsp, r8",
            "mov rbp, r8",
            "jmp rdx",
            in("r8") KERNEL_STACK_BOTTOM,
            in("rcx") kernel_args,
            in("rdx") kernel_wrapper as *const u8,
        )
    };

    unreachable!()
}

// called by assembly
fn kernel_wrapper(kernel_args: *mut KernelArgs) -> ! {
    kernel_main(unsafe { kernel_args.read() });
}
