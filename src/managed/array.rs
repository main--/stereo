use std::{mem, slice};
use std::fmt::{Debug, Result as FmtResult, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index};
use std::os::raw::c_void;
use std::borrow::Borrow;

use super::{Referenceable, GenericObject, Object, Primitive};
use metadata::Class;
use runtime::AppDomain;
use safety::{GcPtrStrategy, BYPASS};
use native;

pub struct ObjectArray(*mut native::MonoArray);

unsafe impl Referenceable for ObjectArray {
    fn ptr(&self) -> *mut native::MonoObject {
        self.0 as *mut _
    }
}
unsafe impl Object for ObjectArray {
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> ObjectArray {
        ObjectArray(ptr as *mut _)
    }
}


impl ObjectArray {
    gc_ret! {
        pub fn new(domain: &AppDomain, class: &Class, length: usize) -> ObjectArray {
            unsafe { ObjectArray(native::mono_array_new(domain.as_raw(), class.as_raw(), length)) }
        }
    }

    /*
    // gc_ret! is too dumb to handle this one :(
    pub fn from_iter<T, B, S>(domain: &AppDomain,
                           class: &Class,
                           iter: T,
                           strat: &S) -> S::Target
        where T: IntoIterator,
    //T::Item: Into<ObjectReference>,
    T::Item: Borrow<B>,
    B: Into<ObjectReference>,
              T::IntoIter: ExactSizeIterator,
              S: GcPtrStrategy<ObjectArray> {
        let iter = iter.into_iter();
        let array = ObjectArray::new(domain, class, iter.len(), BYPASS);
        for (i, x) in iter.enumerate() {
            array.set(i, x.borrow().deref());
        }

        strat.wrap(array)
    }

    // TODO: indexer mb? (nvm we can just deref to slice)
    pub fn get(&self, index: usize) -> Option<ObjectReference> {
        if index >= self.len() { return None; }

        unsafe {
            let ptr = native::mono_array_addr_with_size(self.0, mem::size_of::<usize>() as i32, index);
            Some(ObjectReference::from_raw(*(ptr as *const *mut native::MonoObject)))
        }
    }
     */

    /*
    pub fn set<T: Into<ObjectReference>>(&self,
                                         index: usize,
                                         value: T) {
        assert!(index < self.len());
        unsafe {
            let ptr = native::mono_array_addr_with_size(self.0, mem::size_of::<usize>() as i32, index);
            native::mono_gc_wbarrier_set_arrayref(self.0, ptr as *mut c_void, value.into().raw());
        }
    }
     */

    // gc_ret! is too dumb to handle this one :(
    pub fn from_iter<I, T, S>(domain: &AppDomain,
                              class: &Class,
                              iter: I,
                              strat: &S) -> S::Target
        where S: GcPtrStrategy<ObjectArray>,
              I: IntoIterator,
              I::IntoIter: ExactSizeIterator,
              I::Item: Borrow<T>,
              T: Referenceable,
    {
        let iter = iter.into_iter();
        let array = ObjectArray::new(domain, class, iter.len(), BYPASS);
        for (i, x) in iter.enumerate() {
            array.set(i, x.borrow().deref());
        }

        strat.wrap(array)
    }

    pub fn set<T: Referenceable>(&self,
                                    index: usize,
                                    value: T) {
        assert!(index < self.len());
        unsafe {
            let ptr = native::mono_array_addr_with_size(self.0, mem::size_of::<usize>() as i32, index);
            native::mono_gc_wbarrier_set_arrayref(self.0, ptr as *mut c_void, value.ptr());
        }
    }

    pub fn len(&self) -> usize {
        unsafe { native::mono_array_length(self.0) }
    }
}

impl Deref for ObjectArray {
    type Target = [GenericObject];

    fn deref(&self) -> &[GenericObject] {
        unsafe {
            let ptr = native::mono_array_addr_with_size(self.0, mem::size_of::<usize>() as i32, 0);
            slice::from_raw_parts(ptr as *const GenericObject, self.len())
        }
    }
}



pub struct PrimitiveArray<P: Primitive> {
    array: *mut native::MonoArray,
    contents: PhantomData<*mut P>,
}

unsafe impl<P: Primitive> Referenceable for PrimitiveArray<P> {
    fn ptr(&self) -> *mut native::MonoObject {
        self.array as *mut _
    }
}
unsafe impl<P: Primitive> Object for PrimitiveArray<P> {
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> PrimitiveArray<P> {
        PrimitiveArray {
            array: ptr as *mut _,
            contents: PhantomData,
        }
    }
}


impl<P: Primitive> PrimitiveArray<P> {
    pub fn new(domain: &AppDomain, length: usize) -> PrimitiveArray<P> {
        unsafe {
            let array = native::mono_array_new(domain.as_raw(), P::class_unsafe().as_raw(), length);
            PrimitiveArray {
                array,
                contents: PhantomData,
            }
        }
    }
}

impl<P: Primitive> Deref for PrimitiveArray<P> {
    type Target = [P];

    fn deref(&self) -> &[P] {
        unsafe {
            let len = native::mono_array_length(self.array);
            let addr = native::mono_array_addr_with_size(self.array, mem::size_of::<P>() as i32, 0);
            slice::from_raw_parts(addr as *const P, len)
        }
    }
}

impl<P: Primitive> DerefMut for PrimitiveArray<P> {
    fn deref_mut(&mut self) -> &mut [P] {
        unsafe {
            let len = native::mono_array_length(self.array);
            let addr = native::mono_array_addr_with_size(self.array, mem::size_of::<P>() as i32, 0);
            slice::from_raw_parts_mut(addr as *mut P, len)
        }
    }
}

impl<P: Primitive> Debug for PrimitiveArray<P> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str("Managed")?;
        Debug::fmt(self.deref(), fmt)
    }
}

impl<P: Primitive> Clone for PrimitiveArray<P> {
    fn clone(&self) -> PrimitiveArray<P> {
        unsafe {
            PrimitiveArray {
                array: native::mono_array_clone(self.array),
                contents: PhantomData,
            }
        }
    }
}
