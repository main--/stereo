use std::fmt::{Debug, Result as FmtResult, Formatter};
use std::marker::PhantomData;

use super::*;
use native;

pub struct GcHandle<T: Object> {
    id: u32,
    phantom: PhantomData<T>,
}

impl<T: Object> GcHandle<T> {
    pub fn new(obj: T) -> GcHandle<T> {
        unsafe {
            GcHandle {
                id: native::mono_gchandle_new(obj.ptr(), 0/*false*/),
                phantom: PhantomData,
            }
        }
    }

    pub fn target(&self) -> T {
        unsafe { T::from_ptr(native::mono_gchandle_get_target(self.id)) }
    }
}

impl<T: Object> Drop for GcHandle<T> {
    fn drop(&mut self) {
        unsafe {
            native::mono_gchandle_free(self.id)
        }
    }
}

impl<T: Object> Debug for GcHandle<T> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "GcHandle#{}", self.id)
    }
}

unsafe impl<T: Object> Referencable for GcHandle<T> {
    fn ptr(&self) -> *mut native::MonoObject {
        self.target().ptr()
    }
}
