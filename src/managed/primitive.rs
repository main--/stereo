use std::fmt::Debug;

use metadata::Class;
use runtime::Mono;
use native;

pub unsafe trait Primitive: Debug {
    unsafe fn class_unsafe() -> Class<'static>;

    // This takes a &Mono because these classes only exist
    // once mscorlib is loaded.
    fn class<'a>(_: &'a Mono) -> Class<'a> {
        unsafe { Self::class_unsafe() }
    }
}

unsafe impl Primitive for i32 {
    unsafe fn class_unsafe() -> Class<'static> {
        Class::from_raw(native::mono_get_int32_class())
    }
}
