use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;
use std::marker::PhantomData;

use super::*;
use native;
use managed::{Referencable, Object, Array};
use managed::object::{GenericObject, ObjectReference};
use managed::array::ObjectArray;

pub struct Method<'image> {
    image: PhantomData<&'image Image<'image>>,
    method: *mut native::MonoMethod,
}

impl<'image> Method<'image> {
    pub unsafe fn from_raw(ptr: *mut native::MonoMethod) -> Method<'image> {
        Method {
            image: PhantomData,
            method: ptr,
        }
    }

    pub fn token(&self) -> MethodToken {
        MethodToken(unsafe { native::mono_method_get_token(self.method) })
    }

    pub fn name(&self) -> &'image CStr {
        unsafe { CStr::from_ptr(native::mono_method_get_name(self.method)) }
    }

    pub fn invoke(&self /* this */, params: &[GenericObject/*FIXME*/]) -> Result<GenericObject, GenericObject> {
        let mut exception = ptr::null_mut();
        //let mut args = [ptr::null_mut()]; // [null]
        let mut args = [params[0].0 as *mut c_void];
        unsafe {
            let ret = native::mono_runtime_invoke(self.method,
                                                  ptr::null_mut(), // this
                                                  args.as_mut_ptr(),
                                                  &mut exception);
            if exception.is_null() {
                Ok(GenericObject::from_ptr(ret))
            } else {
                Err(GenericObject::from_ptr(exception))
            }
        }
    }

    pub fn invoke_array<T: Referencable>(&self, this: T, params: &ObjectArray) -> Result</*val*/ObjectReference, /*exception*/GenericObject> {
        // TODO: assert array is of type object[]

        let mut exception = ptr::null_mut();
        unsafe {
            let ret = native::mono_runtime_invoke_array(self.method,
                                                        this.ptr() as *mut _,
                                                        Array::ptr(params),
                                                        &mut exception);
            if exception.is_null() {
                Ok(ObjectReference::from_raw(ret))
            } else {
                Err(GenericObject::from_ptr(exception))
            }
        }
    }
}
