#![no_std]
#![no_main]

use riscv_common::zksync_os_finish_success;

core::arch::global_asm!(include_str!("../../../scripts/asm/asm_reduced.S"));

#[no_mangle]
extern "C" fn eh_personality() {}

#[link_section = ".init.rust"]
#[export_name = "_start_rust"]
unsafe extern "C" fn start_rust() -> ! {
    main()
}

#[export_name = "_setup_interrupts"]
/// # Safety
/// This function is a no-op interrupt setup handler for the zkVM environment.
pub unsafe fn custom_setup_interrupts() {}

#[repr(C)]
#[derive(Debug)]
pub struct MachineTrapFrame {
    pub registers: [u32; 32],
}

#[link_section = ".trap.rust"]
#[export_name = "_machine_start_trap_rust"]
pub extern "C" fn machine_start_trap_rust(_trap_frame: *mut MachineTrapFrame) -> usize {
    unsafe { core::hint::unreachable_unchecked() }
}

#[inline(never)]
fn main() -> ! {
    // Minimal computation - just return 42
    #[allow(unused_unsafe)]
    unsafe { zksync_os_finish_success(&[42, 0, 0, 0, 0, 0, 0, 0]) }
}