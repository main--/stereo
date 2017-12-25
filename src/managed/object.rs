use std::ptr;

use metadata::Class;
use super::{Referencable, Object};
use native;

#[derive(Debug, Clone, Copy)]
pub struct ObjectReference(*mut native::MonoObject);

impl ObjectReference {
    /*
    pub fn null() -> ObjectReference {
        ObjectReference(ptr::null_mut())
    }
     */
    pub fn deref(&self) -> Option<GenericObject> {
        if self.0.is_null() {
            None
        } else {
            Some(GenericObject(self.0))
        }
    }

    pub fn raw(&self) -> *mut native::MonoObject {
        self.0
    }

    pub unsafe fn from_raw(ptr: *mut native::MonoObject) -> ObjectReference {
        ObjectReference(ptr)
    }
}


impl<T: Referencable> From<T> for ObjectReference {
    fn from(obj: T) -> ObjectReference {
        ObjectReference(obj.ptr())
    }
}

impl<T: Referencable> From<Option<T>> for ObjectReference {
    fn from(obj: Option<T>) -> ObjectReference {
        ObjectReference(obj.as_ref().map(Referencable::ptr).unwrap_or(ptr::null_mut()))
    }
}

impl From<ObjectReference> for Option<GenericObject> {
    fn from(obj: ObjectReference) -> Option<GenericObject> {
        obj.deref()
    }
}

#[derive(Debug)]
pub struct GenericObject(pub /*FIXME*/ *mut native::MonoObject);

impl GenericObject {
    pub fn class<'a>(&self) -> Class<'a> {
        unsafe { Class::from_raw(native::mono_object_get_class(self.0)) }
    }

    pub fn is_instance(&self, class: &Class) -> bool {
        unsafe { !native::mono_object_isinst(self.0, class.as_raw()).is_null() }
    }
}

unsafe impl Referencable for GenericObject {
    fn ptr(&self) -> *mut native::MonoObject {
        self.0
    }
}

unsafe impl Object for GenericObject {
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> GenericObject {
        GenericObject(ptr)
    }
}
