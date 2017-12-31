use std::ptr;
use std::marker::PhantomData;
use std::ffi::CString;
use std::ops::Range;

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
unsafe impl<'mono> Sync for Image<'mono> {} // FIXME: smooth out our story here

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

    // TODO: rename Class to Type everywhere
    pub fn classes<'a>(&'a self) -> ClassesIter<'a> {
        unsafe {
            let table = native::mono_image_get_table_info(self.image, native::MonoMetaTableEnum::MONO_TABLE_TYPEDEF as i32);
            let count = native::mono_table_info_get_rows(table) as u32;

            ClassesIter {
                image: self,
                range: 1..count, // skip initial <module> type
            }
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


pub struct ClassesIter<'a> {
    image: &'a Image<'a>,
    range: Range<u32>,
}

impl<'a> Iterator for ClassesIter<'a> {
    type Item = Class<'a>;

    fn next(&mut self) -> Option<Class<'a>> {
        self.range.next()
            .map(|i| TypeToken((i + 1) | native::MonoTokenType::MONO_TOKEN_TYPE_DEF as u32)) // make tokens
            .map(|t| self.image.get_class(t).unwrap())
    }
}
