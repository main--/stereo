use std::fmt::{Debug, Display, Result as FmtResult, Formatter};
use std::hash::{Hash, Hasher};

use widestring::{WideStr, WideString};

use safety::BYPASS;
use runtime::AppDomain;
use super::{Referenceable, Object};
use native;

#[derive(Clone, Copy)] // strings are immutable
pub struct MonoString(*mut native::MonoString);

impl MonoString {
    gc_ret! {
        pub fn new(string: &str, domain: &AppDomain) -> MonoString {
            let wstr = WideString::from_str(string);
            MonoString::new_wide(&wstr, domain, BYPASS)
        }

        pub fn empty(domain: &AppDomain) -> MonoString {
            unsafe { MonoString(native::mono_string_empty(domain.as_raw())) }
        }

        pub fn new_wide(string: &WideStr, domain: &AppDomain) -> MonoString {
            unsafe {
                MonoString(native::mono_string_new_utf16(domain.as_raw(),
                                                         string.as_ptr(),
                                                         string.len() as i32))
            }
        }
    }

    pub fn len(&self) -> usize {
        unsafe { native::mono_string_length(self.0) as usize }
    }

    pub fn text(&self) -> &WideStr {
        unsafe { WideStr::from_ptr(native::mono_string_chars(self.0), self.len()) }
    }
}

impl PartialEq for MonoString {
    fn eq(&self, other: &MonoString) -> bool {
        unsafe { native::mono_string_equal(self.0, other.0) != 0 }
    }
}
impl Eq for MonoString {}
impl Hash for MonoString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(unsafe { native::mono_string_hash(self.0) });
    }
}
impl Debug for MonoString {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "\"{}\"", self.text().to_string_lossy())
    }
}
impl Display for MonoString {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str(&self.text().to_string_lossy())
    }
}

unsafe impl Referenceable for MonoString {
    fn ptr(&self) -> *mut native::MonoObject { self.0 as *mut _ }
}

unsafe impl Object for MonoString {
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> MonoString { MonoString(ptr as *mut _) }
}
