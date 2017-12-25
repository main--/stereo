use std::mem::ManuallyDrop;
use std::ffi::CString;

use metadata::{Image, Class};
use native;

mod appdomain;
mod assembly;
pub use self::appdomain::AppDomain;
pub use self::assembly::Assembly;

pub struct Mono {
    root_domain: ManuallyDrop<AppDomain<'static>>,
    corlib: ManuallyDrop<Image<'static>>,
}

impl Mono {
    pub fn init() -> Option<Mono> {
        unsafe {
            let domain = native::mono_jit_init(cstr!("stereo"));
            if domain.is_null() {
                None
            } else {
                let corlib = native::mono_get_corlib();
                Some(Mono {
                    root_domain: ManuallyDrop::new(AppDomain::from_raw(domain)),
                    corlib: ManuallyDrop::new(Image::from_raw(corlib)),
                })
            }
        }
    }

    pub fn root_domain<'a>(&'a self) -> &'a AppDomain<'a> {
        &self.root_domain
    }

    pub fn create_domain<'a>(&'a self) -> AppDomain<'a> {
        unsafe {
            // FIXME: allow passing these strs
            let domain = native::mono_domain_create_appdomain(cstr!("asd") as *mut _,
                                                              cstr!("nein") as *mut _);
            assert!(!domain.is_null());
            AppDomain::from_raw(domain)
        }
    }

    pub fn mscorlib<'a>(&'a self) -> &'a Image<'a> {
        &self.corlib
    }

    pub fn class_object(&self) -> Class {
        unsafe { Class::from_raw(native::mono_get_object_class()) }
    }

    pub fn class_string(&self) -> Class {
        unsafe { Class::from_raw(native::mono_get_string_class()) }
    }

    pub fn open_image<'a, 'b>(&'a self, path: &'b str) -> Result<Image<'a>, native::MonoImageOpenStatus> {
        // FIXME this type can return Err(OK), this is a bug
        let path = CString::new(path).unwrap();
        unsafe {
            let mut status = native::MonoImageOpenStatus::MONO_IMAGE_OK;
            let image = native::mono_image_open(path.as_ptr(), &mut status);
            match status {
                native::MonoImageOpenStatus::MONO_IMAGE_OK => Ok(Image::from_raw(image)),
                error => Err(error),
            }
        }
    }
}

impl Drop for Mono {
    fn drop(&mut self) {
        panic!("Dropping mono is not allowed!");
    }
}
