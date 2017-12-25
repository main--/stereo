use std::ptr;
use std::marker::PhantomData;

use metadata::Image;
use super::{Mono, Assembly};
use native;

pub struct AppDomain<'mono> {
    mono: PhantomData<&'mono Mono>,
    domain: *mut native::MonoDomain,
}

impl<'mono> AppDomain<'mono> {
    pub unsafe fn from_raw(ptr: *mut native::MonoDomain) -> AppDomain<'mono> {
        AppDomain {
            mono: PhantomData,
            domain: ptr,
        }
    }

    pub fn as_raw(&self) -> *mut native::MonoDomain {
        self.domain
    }

    // todo: this is bad, need reset feature
    fn set(&self) {
        unsafe { native::mono_domain_set(self.domain, 1) }; // TODO: check ret
    }

    pub fn load_assembly(&self, image: &Image) -> Result<Assembly, native::MonoImageOpenStatus> {
        self.set();
        let mut status = native::MonoImageOpenStatus::MONO_IMAGE_OK;
        let assembly = unsafe { native::mono_assembly_load_from(image.as_raw(),
                                                                cstr!("fname"),
                                                                &mut status) };
        if assembly.is_null() {
            Err(status)
        } else {
            Ok(Assembly { domain: self, assembly })
        }
    }

    /*
    pub fn new_string_array(&self) -> Option<ObjectArray> {
        //let esr = ObjectReference::from(Some(unsafe { GenericObject::from_ptr(native::mono_string_empty(self.0) as *mut _) }));
        let array = ObjectArray::new(self, &Class(unsafe { native::mono_get_string_class() }), 2);
        let str1 = MonoString::empty(self);
        let str2 = MonoString::new("PogChamp", self);
        array.set(0, str1);
        array.set(1, str2);
        Some(array)
    }
     */

    /*
    pub fn new_string_array(&self) -> Option<Array> {
        let class = unsafe { native::mono_get_string_class() }; // FIXME everything
        let arr = unsafe { native::mono_array_new(self.0, class, 2) };
        unsafe {
            let empty_str = native::mono_string_new(self.0, cstr!("PogChamp")); //native::mono_string_empty(self.0);

            let ptr = native::mono_array_addr_with_size(arr, 8 / * ptr size * /, 0) as *mut c_void;
            native::mono_gc_wbarrier_set_arrayref(arr, ptr, empty_str as *mut native::MonoObject);
            // *ptr = empty_str;

            let ptr = native::mono_array_addr_with_size(arr, 8 / * ptr size * /, 1) as *mut c_void;
            native::mono_gc_wbarrier_set_arrayref(arr, ptr, empty_str as *mut native::MonoObject);
        }

        wrap_ptr!(arr, Array)
    }
     */

    //pub fn new_primitive_array(&self,

    /*
    pub fn new_int_array(&self) -> Option<PrimitiveArray<i32>> {
        let sp = unsafe {
            let s = native::mono_string_new(self.0, cstr!("wtfbug"));
            (s as usize) ^ 0xabad1dea
        };
        let mut array = PrimitiveArray::new(self, 2);
        array[0] = 42;
        unsafe {
            native::mono_gc_collect(native::mono_gc_max_generation());

            let mystr = (sp ^ 0xabad1dea) as *mut native::MonoString;
            let mystry = native::mono_string_length(mystr);
        }
        array[1] = 1337;
        Some(array)
    }
     */
}

impl<'mono> Drop for AppDomain<'mono> {
    fn drop(&mut self) {
        unsafe {
            let mut exc = ptr::null_mut();
            native::mono_domain_try_unload(self.domain, &mut exc);
            assert!(exc.is_null());
        }
    }
}
