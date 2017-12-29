use std::fmt::{Debug, Result as FmtResult, Formatter};
use std::ptr;

use metadata::Class;
use super::{Referenceable, Object, MonoString};
use native;

#[repr(C)]
pub struct GenericObject(*mut native::MonoObject);

impl GenericObject {
    pub fn class<'a>(&self) -> Class<'a> {
        unsafe { Class::from_raw(native::mono_object_get_class(self.0)) }
    }

    pub fn is_instance(&self, class: &Class) -> bool {
        unsafe { !native::mono_object_isinst(self.0, class.as_raw()).is_null() }
    }
}

unsafe impl Referenceable for GenericObject {
    fn ptr(&self) -> *mut native::MonoObject {
        self.0
    }
}

unsafe impl Object for GenericObject {
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> GenericObject {
        GenericObject(ptr)
    }
}

impl Debug for GenericObject {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        assert!(!self.0.is_null());

        fmt.write_str("")?;
        self.class().fmt(fmt)?;
        fmt.write_str(": ")?;

        unsafe {
            let mut exception = ptr::null_mut();
            let s = native::mono_object_to_string(self.0, &mut exception);
            if exception.is_null() {
                Debug::fmt(&MonoString::from_ptr(s as *mut _), fmt)?;
            } else {
                fmt.write_str("Exception in ToString()")?;
            }
        }

        Ok(())
    }
}
