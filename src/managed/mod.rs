use std::ptr;
use std::marker::PhantomData;

use metadata::Class;
use native;

// These wrap actual managed objects
mod string;
mod array;
pub mod object;
// pub mod exception;
mod boxed;

// These don't, but are still relevant
mod gchandle;
mod weakhandle;
mod value;
mod primitive;

// TODO: sort these?
pub use self::weakhandle::WeakHandle;
pub use self::gchandle::GcHandle;
pub use self::value::MonoValue;
pub use self::string::MonoString;
pub use self::primitive::Primitive;
pub use self::boxed::Boxed;
pub use self::array::Array;

use self::object::{GenericObject};


/// All object references (nullable!).
pub unsafe trait Referenceable {
    fn ptr(&self) -> *mut native::MonoObject;
}

unsafe impl<'a, T: Referenceable> Referenceable for &'a T {
    fn ptr(&self) -> *mut native::MonoObject {
        Referenceable::ptr(*self)
    }
}



/// References to actual objects (not nullable).
pub unsafe trait Object: Referenceable {
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> Self;

    // FIXME: remove
    fn class(&self) -> Class {
        unsafe { Class::from_raw(native::mono_object_get_class(self.ptr())) }
    }

    // FIXME: remove
    fn downcast(self) -> GenericObject where Self: Sized {
        unsafe { GenericObject::from_ptr(self.ptr()) }
    }
}





#[repr(C)]
#[derive(Debug)]
pub struct Nullable<T: Object> {
    ptr: *mut native::MonoObject,
    p: PhantomData<T>,
}

impl<T: Object> Nullable<T> {
    pub unsafe fn from_raw(ptr: *mut native::MonoObject) -> Nullable<T> {
        Nullable { ptr, p: PhantomData }
    }
}

unsafe impl<T: Object> Referenceable for Nullable<T> {
    fn ptr(&self) -> *mut native::MonoObject {
        self.ptr
    }
}

impl<T: Object> From<Option<T>> for Nullable<T> {
    fn from(o: Option<T>) -> Nullable<T> {
        unsafe {
            Nullable::from_raw(o.as_ref().map(Referenceable::ptr).unwrap_or(ptr::null_mut()))
        }
    }
}

pub type ObjectReference = Nullable<GenericObject>;





#[derive(Clone, Copy, Debug)]
pub struct Null;

unsafe impl Referenceable for Null {
    fn ptr(&self) -> *mut native::MonoObject { ptr::null_mut() }
}



pub unsafe trait StaticallyTyped {
    // TODO: this can be safe
    unsafe fn class() -> Class<'static>;

    const IS_REFERENCE: bool;
}

/*
pub unsafe trait Array: Object {
    fn ptr(&self) -> *mut native::MonoArray { Referencable::ptr(self) as *mut _ }
}
 */
