use std::mem;
use std::slice;
use std::os::raw::c_void;

mod cherenkov;
use cherenkov::{Config, nova};

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut c_void, cap: usize) {
    unsafe  {
        let _buf = Vec::from_raw_parts(ptr, 0, cap);
    }
}

#[no_mangle]
pub extern "C" fn fill(pointer: *mut u8, width: usize, height: usize, x: f64, y: f64, r: f64, g: f64, b: f64, n_spokes: usize, radius: f64, random_hue: f64) {
    let config = Config {
        center: (x, y),
        color: (r, g, b),
        n_spokes,
        radius,
        random_hue
    };
    let rowstride = width * 4;
    let mut buffer = unsafe { slice::from_raw_parts_mut(pointer, width * height * 4) };
    nova(&config, &mut buffer, rowstride as i32, width as i32, height as i32, 2);
}
