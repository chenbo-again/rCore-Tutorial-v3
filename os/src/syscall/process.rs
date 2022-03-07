

use crate::config::{PAGE_SIZE};
use crate::mm::{VirtAddr, PhysAddr, VirtPageNum, MapPermission};
use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next, current_translate, current_mmap, current_munmap,
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
    // 检查port是否合法
    if port & !0x7 != 0 ||
    port & 0x7 == 0 ||
    // 检查 start 对齐
    start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    // 向上取整
    let page_len = if len % PAGE_SIZE != 0 {
        len / PAGE_SIZE + 1
    } else {
        len / PAGE_SIZE
    };

    // 检查这块内存是否被申请过
    for p_index in 0..page_len {
        let vpn: VirtPageNum = VirtAddr::from(p_index * PAGE_SIZE + start).into();
        if current_translate(vpn).is_some() {
            return -1;
        }
    }

    let mpp: MapPermission = if port & 1 != 0 {MapPermission::R} else {MapPermission::empty()} |
                            if port & 2 != 0 {MapPermission::W} else {MapPermission::empty()}  |
                            if port & 4 != 0 {MapPermission::X} else {MapPermission::empty()}  |
                            MapPermission::U;
    
    // R == 0 && W == 1 is illegal in riscv
    if mpp.contains(MapPermission::W) && !mpp.contains(MapPermission::R) {
        return -1;
    }
    
    for p_index in 0..page_len {
        // 这里检查是否申请成功
        let vpn = VirtAddr::from(p_index * PAGE_SIZE + start).into();
        if !current_mmap(vpn, mpp) {
            return -1;
        }
    }

    0
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    
    // 向上取整
    let page_len = if len % PAGE_SIZE != 0 {
        len / PAGE_SIZE + 1
    } else {
        len / PAGE_SIZE
    };

    // 检查这块内存是否被申请过
    for p_index in 0..page_len {
        let vpn: VirtPageNum = VirtAddr::from(p_index * PAGE_SIZE + start).into();
        if current_translate(vpn).is_none() {
            return -1;
        }
    }

    for p_index in 0..page_len {
        let vpn = VirtAddr::from(p_index * PAGE_SIZE + start).into();
        current_munmap(vpn);
    }
    
    0
}