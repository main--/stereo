use std::ffi::CString;

use super::AppDomain;
use native;

// FIXME: all of this
pub struct Assembly<'domain> {
    pub domain: &'domain AppDomain<'domain>,
    pub assembly: *mut native::MonoAssembly,
}

impl<'domain> Assembly<'domain> {
    /*
    pub unsafe fn from_raw(ptr: *mut native::MonoAssembly) -> Assembly<'domain> {
        Assembly {
            domain:
        }
    }

    pub fn execute<I>(&self, args: I) -> i32 where I: IntoIterator, I::Item: AsRef<str> {
        let mut args: Vec<_> = args.into_iter()
            .map(|s| CString::new(s.as_ref()).unwrap().into_raw()).collect();
        assert!(!args.is_empty(), "Must pass at least program basename!");

        // TODO fix cast integer overflow
        unsafe {
            let ret = native::mono_jit_exec(self.domain.as_raw(),
                                            self.assembly,
                                            args.len() as i32,
                                            args.as_mut_ptr());
            for cs in args {
                drop(CString::from_raw(cs));
            }
            ret
        }
    }
     */
}
