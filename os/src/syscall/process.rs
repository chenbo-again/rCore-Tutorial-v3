

use crate::mm::{VirtAddr, PhysAddr};
use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next, current_translate,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let vaddr = VirtAddr::from(_ts as usize);
    match current_translate(vaddr.floor()) {
        Some(pte) => {
            if pte.writable() {
                let ts = (PhysAddr::from(pte.ppn()).0 + vaddr.page_offset()) as *mut TimeVal;
                unsafe {
                    *ts = TimeVal {
                        sec: us / 1_000_000,
                        usec: us % 1_000_000,
                    };
                }
                0
            } else {
                -1
            }
        },
        None => -1,
    }
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    todo!()
}