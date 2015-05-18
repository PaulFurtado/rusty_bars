#![allow(unstable)]

extern crate libc;
use self::libc::{size_t, c_void};
use std::{mem, slice};
use fftw::ext;


/// Wrapper around fftw_malloc which automatically allocates the right amount of
/// space for Rust objects, similar to calloc.
/// Returns None if allocation failed.
fn fftw_ralloc<T>(count: usize) -> Option<*mut T> {
    let element_size = mem::size_of::<T>();
    let total_size = element_size * count;
    let result: *mut T = unsafe { ext::fftw_malloc(total_size as size_t) } as *mut T;
    if result.is_null() {
        None
    } else {
        Some(result)
    }
}


/// FFTW depends on memory alignment in order to take advantage of SIMD
/// instructions. While this isn't a massive FFT, alignment is still important
/// because when FFTW chooses the algorithm to use, it needs to consider the
/// alignment. Ex: an algorithm with lots of SIMD instructions would be
/// unusable on unaligned data. Since we have multiple audio channels to run
/// FFTs on, FFTW can plan once and operate on many different arrays if they
/// are all aligned the exact same way.
/// See: http://www.fftw.org/doc/Memory-Allocation.html
/// FftAlignedArray is a type which utilizes FFTW's malloc to take advantage of
/// alignment. It may be possible to use Vec::from_raw_parts, but you need to
/// run FFTW's free function when you're done with it and Vec frees its pointer
/// when it is dropped so stopping that would involve hacks.
/// The FftAlignedArray struct doesn't implement any features a Vec does,
/// instead, it just gives you back slices so you can do
pub struct FftwAlignedArray<T> {
    len: usize,
    ptr: *const T,
    mut_ptr: *mut T,
}

impl<T: Copy> FftwAlignedArray<T> {
    /// Create a new FftwAlignedArray.
    /// Len is the number of elements, not the size in bytes.
    /// Panics if memory allocation fails.
    pub fn new(len: usize) -> FftwAlignedArray<T> {
        let ptr: *mut T = fftw_ralloc::<T>(len).unwrap();
        FftwAlignedArray {
            len: len,
            ptr: ptr as *const T,
            mut_ptr: ptr
        }
    }

    /// Initialize every element in the array with init_val
    pub fn initialize(&mut self, init_val: T) {
        for val in self.as_mut_slice().iter_mut() {
            *val = init_val;
        }
    }

    /// Get an immutable raw pointer to the memory backing this array
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Get an mutable raw pointer to the memory backing this array
    pub fn as_mut_ptr(&self) -> *mut T {
        self.mut_ptr
    }

    /// Modify the contents of this array via a mutable slice
    pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [T] {
        unsafe{ slice::from_raw_parts_mut(&self.mut_ptr, self.len) }
    }

    /// Access the contents of this array via an immutable slice
    fn as_slice(&self) -> &[T] {
        unsafe{ slice::from_raw_parts(&self.ptr, self.len) }
    }
}


#[unsafe_destructor]
/// Unsafe because it has lifetimes.
impl<T> Drop for FftwAlignedArray<T> {
    /// Free the array with the right deallocator
    fn drop(&mut self) {
        unsafe{ ext::fftw_free(self.mut_ptr as *mut c_void) };
    }
}
