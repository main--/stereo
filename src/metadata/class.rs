use std::ffi::CStr;
use std::ptr;
use std::fmt::{Debug, Result as FmtResult, Formatter};
use std::os::raw::c_void;
use std::marker::PhantomData;
use std::borrow::Cow;

use super::*;
use native;

#[derive(Clone)]
pub struct Class<'image> {
    image: PhantomData<&'image Image<'image>>,
    class: *mut native::MonoClass,
}

impl<'image> Class<'image> {
    pub unsafe fn from_raw(ptr: *mut native::MonoClass) -> Class<'image> {
        Class {
            image: PhantomData,
            class: ptr,
        }
    }

    pub fn as_raw(&self) -> *mut native::MonoClass {
        self.class
    }

    // TODO: naming
    pub fn outer(&self) -> Option<Class<'image>> {
        unsafe {
            let outer = native::mono_class_get_nesting_type(self.class);
            wrap_ptr!(outer, Class)
        }
    }

    pub fn namespace(&self) -> Option<&'image str> {
        self.namespace_strict().or_else(|| self.outer().and_then(|x| x.namespace()))
    }

    pub fn namespace_strict(&self) -> Option<&'image str> {
        unsafe {
            let ns = native::mono_class_get_namespace(self.class);
            if *ns == 0 {
                None
            } else {
                Some(CStr::from_ptr(ns).to_str().unwrap())
            }
        }
    }

    pub fn name(&self) -> &'image str {
        unsafe {
            CStr::from_ptr(native::mono_class_get_name(self.class)).to_str().unwrap()
        }
    }

    pub fn token(&self) -> TypeToken {
        TypeToken(unsafe { native::mono_class_get_type_token(self.class) })
    }

    pub fn methods(&self) -> ClassMethodsIter<'image> {
        ClassMethodsIter {
            class: self.clone(),
            iter: ptr::null_mut(),
            index: 0,
        }
    }
}

pub struct ClassMethodsIter<'image> {
    class: Class<'image>,
    iter: *mut c_void,
    index: usize,
}


impl<'image> Iterator for ClassMethodsIter<'image> {
    type Item = Method<'image>;

    fn next(&mut self) -> Option<Method<'image>> {
        unsafe {
            let method = native::mono_class_get_methods(self.class.class, &mut self.iter);
            let res = wrap_ptr!(method, Method);
            if res.is_some() {
                self.index += 1;
            }
            res
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // FIXME: conversion integer overflow?
        let intlen = unsafe { native::mono_class_num_methods(self.class.class) };
        let size = (intlen as usize) - self.index;
        (size, Some(size))
    }

    fn count(self) -> usize {
        self.size_hint().0
    }
}

impl<'image> ExactSizeIterator for ClassMethodsIter<'image> {}



impl<'image> Debug for Class<'image> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        if let Some(outer) = self.outer() {
            outer.fmt(fmt)?;
            fmt.write_str("/")?;
        }

        if let Some(ns) = self.namespace_strict() {
            fmt.write_str(ns)?;
            fmt.write_str(".")?;
        }

        fmt.write_str(self.name().as_ref())
    }
}
