#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(slice_ptr_get)]
use std::alloc::*;
use std::marker::PhantomData;
use std::ops::*;
use std::ptr::*;

pub struct SmolVec<T, A: Allocator = Global>(*mut u8, PhantomData<T>, A);

impl<T> SmolVec<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T> Default for SmolVec<T> {
    fn default() -> Self {
        Self(
            unsafe { alloc_zeroed(Self::layout(0)) },
            PhantomData,
            Global,
        )
    }
}

impl<T, A: Allocator> SmolVec<T, A> {
    pub fn new_in(alloc: A) -> Self {
        Self(unsafe { alloc_zeroed(Self::layout(0)) }, PhantomData, alloc)
    }

    const USIZE: usize = std::mem::size_of::<usize>();

    pub fn len_ptr_mut(&self) -> *mut usize {
        Self::len_from_ptr(self.0)
    }

    pub fn len(&self) -> usize {
        unsafe { *self.len_ptr_mut() }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn cap_ptr_mut(&self) -> *mut usize {
        Self::cap_from_ptr(self.0)
    }

    pub fn cap(&self) -> usize {
        unsafe { *self.cap_ptr_mut() }
    }

    pub fn push(&mut self, t: T) {
        if self.len() == self.cap() {
            unsafe {
                self.resize();
            }
        }

        let len = self.len_ptr_mut();
        unsafe {
            let ptr = self.as_mut_ptr().add(*len);
            write(ptr, t);
            *len += 1;
        }
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        Self::mut_ptr(self.0)
    }

    fn as_ptr(&self) -> *const T {
        Self::mut_ptr(self.0) as *const T
    }

    fn mut_ptr(ptr: *mut u8) -> *mut T {
        unsafe { ptr.add(2 * Self::USIZE) as *mut _ }
    }

    fn len_from_ptr(ptr: *mut u8) -> *mut usize {
        ptr as *mut _
    }

    fn cap_from_ptr(ptr: *mut u8) -> *mut usize {
        unsafe { ptr.add(Self::USIZE) as *mut _ }
    }

    fn layout(cap: usize) -> Layout {
        Layout::new::<(usize, usize)>()
            .extend(Layout::new::<T>().repeat(cap).unwrap().0)
            .unwrap()
            .0
    }

    unsafe fn resize(&mut self) {
        println!("resize");
        let cap = self.cap_ptr_mut();
        let new_cap = if *cap == 0 { 1 } else { *cap * 2 };

        self.0 = self
            .2
            .grow(
                NonNull::new_unchecked(self.0),
                Self::layout(*cap),
                Self::layout(new_cap),
            )
            .unwrap()
            .as_mut_ptr();
        *cap = new_cap
    }
}

impl<T, A: Allocator> Drop for SmolVec<T, A> {
    fn drop(&mut self) {
        let s: &mut [T] = self;
        unsafe {
            drop_in_place(s);
            self.2
                .deallocate(NonNull::new_unchecked(self.0), Self::layout(self.cap()));
        }
    }
}

impl<T, A: Allocator> Deref for SmolVec<T, A> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
    }
}

impl<T, A: Allocator> DerefMut for SmolVec<T, A> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::SmolVec;

    #[test]
    fn test() {
        let mut vec = SmolVec::<usize>::new();
        println!("{:?}", vec.deref());
        for i in 0..32 {
            vec.push(i);
            println!("{:?}", vec.deref());
        }
    }
}
