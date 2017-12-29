use std::fmt::{Debug, Result as FmtResult, Formatter};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;

use super::*;
use safety::{StackRefs, BYPASS};
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

    pub fn pin(&self) -> PinnedGcHandle<T> {
        PinnedGcHandle::new(self.target(BYPASS))
    }

    // this hands out a stackref, so you need to accept the t&c for that
    pub fn target(&self, _: &StackRefs) -> T {
        unsafe { T::from_ptr(native::mono_gchandle_get_target(self.id)) }
    }

    pub fn downcast(self) -> GcHandle<GenericObject> {
        let id = self.id;
        mem::forget(self);
        GcHandle { id, phantom: PhantomData }
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

impl<T: Object> Clone for GcHandle<T> {
    fn clone(&self) -> GcHandle<T> {
        GcHandle::new(self.target(BYPASS))
    }
}

unsafe impl<T: Object> Referenceable for GcHandle<T> {
    fn ptr(&self) -> *mut native::MonoObject {
        // pointers are harmless (not our responsibility)
        // thus bypass is safe
        self.target(BYPASS).ptr()
    }
}

#[derive(Debug)]
pub struct PinnedGcHandle<T: Object> {
    handle: GcHandle<T>,
    pin: T,
}

impl<T: Object> PinnedGcHandle<T> {
    pub fn new(obj: T) -> PinnedGcHandle<T> {
        unsafe {
            PinnedGcHandle {
                handle: GcHandle {
                    id: native::mono_gchandle_new(obj.ptr(), 1/*true*/),
                    phantom: PhantomData,
                },
                pin: obj,
            }
        }
    }
}

impl<T: Object> Deref for PinnedGcHandle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.pin
    }
}

impl<T: Object> Clone for PinnedGcHandle<T> {
    fn clone(&self) -> PinnedGcHandle<T> {
        PinnedGcHandle::new(self.handle.target(BYPASS))
    }
}
