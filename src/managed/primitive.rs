use std::fmt::Debug;

use super::StaticallyTyped;
use metadata::Class;
use runtime::Mono;
use native;

pub unsafe trait Primitive: Debug + StaticallyTyped {
    //unsafe fn class_unsafe() -> Class<'static>;

    /*
    // This takes a &Mono because these classes only exist
    // once mscorlib is loaded.
    fn class<'a>(_: &'a Mono) -> Class<'a> {
        unsafe { Self::class_unsafe() }
    }
     */
}

macro_rules! impl_primitives {
    ( $( $tyrust:ident : $tymono:ident ),+ ) => {
        $(
            unsafe impl StaticallyTyped for $tyrust {
                unsafe fn class() -> Class<'static> {
                    Class::from_raw( native:: $tymono () )
                }

                const IS_REFERENCE: bool = false;
            }
            unsafe impl Primitive for $tyrust {}
        )+
    }
}

impl_primitives! {
    i32: mono_get_int32_class,
    i64: mono_get_int64_class
}
