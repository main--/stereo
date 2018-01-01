use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::{Referenceable, Object, Primitive};
use runtime::AppDomain;
use native;

// TODO: support value types, like, at all
pub struct Boxed<T: Primitive> {
    ptr: *mut native::MonoObject,
    phantom: PhantomData<T>,
}

impl<T: Primitive> Boxed<T> {
    gc_ret! {
        pub fn new(domain: &AppDomain, t: T) -> Boxed<T> {
            unsafe {
                let myptr: *const T = &t;
                let ptr = native::mono_value_box(domain.as_raw(),
                                                 T::class().as_raw(),
                                                 myptr as *mut _);

                Boxed {
                    ptr,
                    phantom: PhantomData,
                }
            }
        }
    }

    // TODO: remove this
    pub fn cast<O: Object>(t: &O) -> Boxed<T> {
        unsafe {
            assert_eq!(t.class().as_raw(), T::class().as_raw());
            Boxed { ptr: t.ptr(), phantom: PhantomData }
        }
    }
}

impl<T: Primitive> Deref for Boxed<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &*(native::mono_object_unbox(self.ptr) as *mut T)
        }
    }
}

unsafe impl<T: Primitive> Referenceable for Boxed<T> {
    fn ptr(&self) -> *mut native::MonoObject {
        self.ptr
    }
}

unsafe impl<T: Primitive> Object for Boxed<T> {
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> Boxed<T> {
        Boxed { ptr, phantom: PhantomData }
    }
}
