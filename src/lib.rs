#[macro_use] extern crate cstr_macro;
extern crate widestring;

mod native {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

macro_rules! wrap_ptr {
    ($eptr:expr, $wrap:ident) => (
        {
            let ptr = $eptr ;
            if ptr.is_null() {
                None
            } else {
                Some ( $wrap ::from_raw ( ptr ) )
            }
        }
    )
}


macro_rules! gc_ret {
    ( $( pub fn $fname:ident ( $( $pname:ident : $ptype:ty ),* ) -> $ret:ty $body:block )+ ) => {
        $(
            pub fn $fname <S: ::safety::GcPtrStrategy < $ret >> (
                $( $pname : $ptype ),* , strat: &S
                    ) -> S::Target {
                strat.wrap( $body )
            }
        )+
    }
}


pub mod managed;
pub mod metadata;
pub mod safety;
pub mod runtime;
