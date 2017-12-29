use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;
use std::marker::PhantomData;
use std::fmt::{Debug, Result as FmtResult, Formatter};
use std::borrow::Cow;

use super::*;
use native;
use safety::GcPtrStrategy;
use managed::{Referenceable, Object, MonoValue, ObjectReference, Primitive};
use managed::object::{GenericObject}; //, ObjectReference};
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

    pub fn name(&self) -> &'image str {
        unsafe {
            CStr::from_ptr(native::mono_method_get_name(self.method)).to_str().unwrap()
        }
    }

    pub fn class(&self) -> Class<'image> {
        unsafe {
            let class = native::mono_method_get_class(self.method);
            assert!(!class.is_null());
            Class::from_raw(class)
        }
    }

    // TODO: wtf is "explicit this" ???

    pub fn is_static(&self) -> bool {
        unsafe {
            let sig = native::mono_method_signature(self.method);
            native::mono_signature_is_instance(sig) == 0
        }
    }

    pub fn return_type(&self) -> Class<'image> {
        unsafe {
            let sig = native::mono_method_signature(self.method);
            let typ = native::mono_signature_get_return_type(sig);
            Class::from_raw(native::mono_class_from_mono_type(typ))
        }
    }

    pub fn parameters(&self) -> MethodParamsIter<'image> {
        MethodParamsIter {
            image: self.image,
            sig: unsafe { native::mono_method_signature(self.method) },
            iter: ptr::null_mut(),
            index: 0,
        }
    }

    pub fn invoke<S>(&self,
                     this: Option<GenericObject>, // FIXME: mb support value types?
                     params: &[MonoValue<S>],
                     strat: &S) -> Result<Option<S::Target>, S::Target>
        where S: GcPtrStrategy<GenericObject>
    {
        let argtypes = self.parameters();
        let argcount = argtypes.len();

        unsafe {
            let mut args: Vec<_> = argtypes.zip(params).map(|(typ, val)| {
                let (result, valtype) = match *val {
                    MonoValue::I32(x) => (x as *mut c_void, i32::class_unsafe()),
                    MonoValue::ObjectRef(Some(ref x)) =>
                        (x.ptr() as *mut c_void, GenericObject::from_ptr(x.ptr()).class()),
                    MonoValue::ObjectRef(None) => return ptr::null_mut(),

                };
                assert!(native::mono_class_is_assignable_from(
                    typ.as_raw(), valtype.as_raw()) != 0, "Invalid parameter type");
                result
            }).collect();
            assert_eq!(args.len(), argcount, "Missing arguments!");

            let this = match this {
                None => {
                    assert!(self.is_static(), "Attempted to call instance method on null!");
                    ptr::null_mut()
                }
                Some(this) => {
                    assert!(!self.is_static(), "Attempted to call a static method on an object!");
                    assert!(native::mono_class_is_assignable_from(
                        self.class().as_raw(), this.class().as_raw()) != 0, "Invalid this type!");

                    this.ptr() as *mut c_void
                }
            };

            let mut exception = ptr::null_mut();
            let ret = native::mono_runtime_invoke(self.method,
                                                  this,
                                                  args.as_mut_ptr(),
                                                  &mut exception);

            if exception.is_null() {
                if ret.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(strat.wrap(GenericObject::from_ptr(ret))))
                }
            } else {
                Err(strat.wrap(GenericObject::from_ptr(exception)))
            }
        }
    }

    // TODO: rework all of this
    #[deprecated]
    pub fn invoke_array<T: Referenceable>(&self, this: T, params: &ObjectArray) -> Result<ObjectReference, GenericObject> {
        // TODO: assert array is of type object[]

        let mut exception = ptr::null_mut();
        unsafe {
            let ret = native::mono_runtime_invoke_array(self.method,
                                                        this.ptr() as *mut _,
                                                        params.ptr() as *mut _,
                                                        &mut exception);
            if exception.is_null() {
                Ok(ObjectReference::from_raw(ret))
            } else {
                Err(GenericObject::from_ptr(exception))
            }
        }
    }
}

impl<'image> Debug for Method<'image> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        unsafe {
            let sig = native::mono_method_signature(self.method);
            let ret = native::mono_signature_get_return_type(sig);

            // return type
            let desc = native::mono_type_full_name(ret);
            {
                let cstr = CStr::from_ptr(desc);
                fmt.write_str(cstr.to_string_lossy().as_ref())?;
            }
            native::g_free(desc as *mut _);
            fmt.write_str(" ")?;

            // class name
            write!(fmt, "{:?}", self.class())?;
            fmt.write_str(".")?;

            // method name
            fmt.write_str(self.name().as_ref())?;

            // params
            fmt.write_str("(")?;
            let desc = native::mono_signature_get_desc(sig, 1/*true*/);
            {
                let cstr = CStr::from_ptr(desc);
                fmt.write_str(cstr.to_string_lossy().as_ref())?;
            }
            native::g_free(desc as *mut _);
            fmt.write_str(")")?;
            Ok(())
        }
    }
}


pub struct MethodParamsIter<'image> {
    image: PhantomData<&'image Image<'image>>,
    sig: *mut native::MonoMethodSignature,
    iter: *mut c_void,
    index: usize,
}

impl<'image> Iterator for MethodParamsIter<'image> {
    type Item = Class<'image>;

    fn next(&mut self) -> Option<Class<'image>> {
        unsafe {
            let param_t = native::mono_signature_get_params(self.sig, &mut self.iter);
            if param_t.is_null() { return None; }
            let param = native::mono_class_from_mono_type(param_t);
            self.index += 1;
            Some(Class::from_raw(param))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // FIXME: conversion integer overflow
        let intlen = unsafe { native::mono_signature_get_param_count(self.sig) };
        let size = (intlen as usize) - self.index;
        (size, Some(size))
    }

    fn count(self) -> usize {
        self.size_hint().0
    }
}
impl<'image> ExactSizeIterator for MethodParamsIter<'image> {}
