use std::ptr;
use std::marker::PhantomData;
use std::ffi::CString;

use runtime::Mono;
use super::*;
use native;

/// TODO
///
/// Images are reference-counted. Cloning them is cheap.
/// The actual image is only unloaded once the last reference dies.
pub struct Image<'mono> {
    mono: PhantomData<&'mono Mono>,
    image: *mut native::MonoImage,
}

impl<'mono> Image<'mono> {
    /// This takes ownership of the reference (you must not close it yourself).
    pub unsafe fn from_raw(ptr: *mut native::MonoImage) -> Image<'mono> {
        Image {
            mono: PhantomData,
            image: ptr,
        }
    }

    pub fn as_raw(&self) -> *mut native::MonoImage {
        self.image
    }

    pub fn into_raw(self) -> *mut native::MonoImage {
        self.image
    }

    pub fn get_method<'a>(&'a self, token: MethodToken) -> Option<Method<'a>> {
        unsafe { wrap_ptr!(native::mono_get_method(self.image, token.0, ptr::null_mut()), Method) }
    }

    pub fn get_class<'a>(&'a self, token: TypeToken) -> Option<Class<'a>> {
        unsafe { wrap_ptr!(native::mono_class_get(self.image, token.0), Class) }
    }

    pub fn class_from_name<'a>(&'a self, namespace: Option<&str>, name: &str) -> Option<Class<'a>> {
        let namespace = CString::new(namespace.unwrap_or("")).unwrap();
        let name = CString::new(name).unwrap();
        unsafe {
            wrap_ptr!(native::mono_class_from_name(self.image,
                                                   namespace.as_ptr(),
                                                   name.as_ptr()), Class)
        }
    }
}

impl<'mono> Clone for Image<'mono> {
    fn clone(&self) -> Image<'mono> {
        unsafe {
            native::mono_image_addref(self.image);
            Image {
                mono: PhantomData,
                image: self.image,
            }
        }
    }
}

impl<'mono> Drop for Image<'mono> {
    fn drop(&mut self) {
        unsafe { native::mono_image_close(self.image) };
    }
}
