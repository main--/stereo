use std::ptr;

use native;

// These wrap actual managed objects
pub mod string;
pub mod array;
pub mod object;
// pub mod exception;
// pub mod boxing;

// These don't, but are still relevant
mod gchandle;
pub mod primitive;

pub use self::gchandle::GcHandle;
use self::object::{GenericObject, ObjectReference};
use self::primitive::MonoPrimitive;

pub unsafe trait Object: Referencable {
    //fn ptr(&self) -> *mut native::MonoObject;
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> Self;

    fn downcast(&self) -> GenericObject {
        unsafe { GenericObject::from_ptr(self.ptr()) }
    }
}

/*
unsafe impl<'a, T: Object> Object for &'a T {
    fn ptr(&self) -> *mut native::MonoObject { T::ptr(*self) as *mut _ }
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> Self { unimplemented!() }
}*/

pub struct Null;

unsafe impl Referencable for Null {
    fn ptr(&self) -> *mut native::MonoObject { ptr::null_mut() }
}

pub unsafe trait Referencable {
    fn ptr(&self) -> *mut native::MonoObject;
}

unsafe impl<'a, T: Referencable> Referencable for &'a T {
    fn ptr(&self) -> *mut native::MonoObject {
        Referencable::ptr(*self)
    }
}


pub unsafe trait Array: Object {
    fn ptr(&self) -> *mut native::MonoArray { Referencable::ptr(self) as *mut _ }
    //unsafe fn from_ptr(ptr: *mut native::MonoArray) -> Self { Object::from_ptr(ptr as *mut _) }
}

/*
unsafe impl<T: Array> Object for  {
    fn ptr(&self) -> *mut native::MonoObject { Array::ptr(self) as *mut _ }
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> Self { Array::from_ptr(ptr as *mut _) }
}
*/
