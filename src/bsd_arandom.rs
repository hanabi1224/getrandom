// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for FreeBSD and NetBSD
use crate::{util_libc::sys_fill_exact, Error};
use core::ptr;

fn kern_arnd(buf: &mut [u8]) -> libc::ssize_t {
    static MIB: [libc::c_int; 2] = [libc::CTL_KERN, libc::KERN_ARND];
    let mut len = buf.len();
    let ret = unsafe {
        libc::sysctl(
            MIB.as_ptr(),
            MIB.len() as libc::c_uint,
            buf.as_mut_ptr() as *mut _,
            &mut len,
            ptr::null(),
            0,
        )
    };
    if ret == -1 {
        -1
    } else {
        len as libc::ssize_t
    }
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // getrandom(2) was introduces in FreeBSD 12.0 and NetBsd 10.0
    #[cfg(target_os = "freebsd")]
    {
        use crate::util_libc::Weak;
        static GETRANDOM: Weak = unsafe { Weak::new("getrandom\0") };
        type GetRandomFn =
            unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::ssize_t;

        if let Some(fptr) = GETRANDOM.ptr() {
            let func: GetRandomFn = unsafe { core::mem::transmute(fptr) };
            return sys_fill_exact(dest, |buf| unsafe { func(buf.as_mut_ptr(), buf.len(), 0) });
        }
    }
    // Both FreeBSD and NetBSD will only return up to 256 bytes at a time, and
    // older NetBSD kernels will fail on longer buffers.
    for chunk in dest.chunks_mut(256) {
        sys_fill_exact(chunk, kern_arnd)?
    }
    Ok(())
}
