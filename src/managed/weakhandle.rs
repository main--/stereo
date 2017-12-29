use std::fmt::{Debug, Result as FmtResult, Formatter};
use std::marker::PhantomData;

use super::*;
use safety::{GcPtrStrategy, BYPASS};
use native;

pub struct WeakHandle<T: Object> {
    id: u32,
    phantom: PhantomData<T>,
}

impl<T: Object> WeakHandle<T> {
    pub fn new(obj: T, track_resurrection: bool) -> WeakHandle<T> {
        unsafe {
            WeakHandle {
                id: native::mono_gchandle_new_weakref(obj.ptr(), track_resurrection as i32),
                phantom: PhantomData,
            }
        }
    }

    pub fn get<S: GcPtrStrategy<T>>(&self, strat: &S) -> Option<S::Target> {
        unsafe {
            let ptr = native::mono_gchandle_get_target(self.id);
            if ptr.is_null() {
                None
            } else {
                Some(strat.wrap(T::from_ptr(ptr)))
            }
        }
    }
}

impl<T: Object> Drop for WeakHandle<T> {
    fn drop(&mut self) {
        unsafe {
            native::mono_gchandle_free(self.id)
        }
    }
}

impl<T: Object> Debug for WeakHandle<T> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "WeakHandle#{}", self.id)
    }
}

unsafe impl<T: Object> Referenceable for WeakHandle<T> {
    fn ptr(&self) -> *mut native::MonoObject {
        unsafe { native::mono_gchandle_get_target(self.id) }
    }
}
