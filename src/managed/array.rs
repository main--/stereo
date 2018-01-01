use std::{mem, slice};
use std::fmt::{Debug, Result as FmtResult, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index};
use std::os::raw::c_void;
use std::borrow::Borrow;

use super::{Referenceable, GenericObject, Object, Primitive, StaticallyTyped};
use metadata::Class;
use runtime::AppDomain;
use safety::{GcPtrStrategy, BYPASS};
use native;


pub struct Array<T: 'static> {
    array: *mut native::MonoArray,
    contents: PhantomData<&'static [T]>,
}

impl<T> Array<T> {
    pub fn len(&self) -> usize {
        unsafe { native::mono_array_length(self.array) }
    }
}

impl Array<GenericObject> {
    gc_ret! {
        pub fn new(domain: &AppDomain, length: usize, class: &Class) -> Array<GenericObject> {
            unsafe {
                let array = native::mono_array_new(domain.as_raw(), class.as_raw(), length);
                Array {
                    array,
                    contents: PhantomData,
                }
            }
        }
    }
}

impl<A> Array<A> {
    // FIXME: figure out whether this actually works for things that aren't object references
    // FIXME 2: typecheck
    pub fn set<T: Referenceable>(&self,
                                 index: usize,
                                 value: T) {
        assert!(index < self.len(), "Array index out of bounds");
        unsafe {
            let ptr = native::mono_array_addr_with_size(self.array, mem::size_of::<usize>() as i32, index);
            native::mono_gc_wbarrier_set_arrayref(self.array, ptr as *mut c_void, value.ptr());
        }
    }
}

impl<P: StaticallyTyped> Array<P> {
    gc_ret! {
        pub fn new(domain: &AppDomain, length: usize) -> Array<P> {
            let class = unsafe { P::class() };
            let array = Array::<GenericObject>::new(domain, length, &class, BYPASS);
            unsafe { mem::transmute(array) }
        }
    }

    // gc_ret! is too dumb to handle this one :(
    pub fn from_iter<I, S>(domain: &AppDomain,
                              iter: I,
                              strat: &S) -> S::Target
        where S: GcPtrStrategy<Array<P>>,
              I: IntoIterator,
              I::IntoIter: ExactSizeIterator,
              I::Item: Borrow<P>,
              P: Referenceable // fix this bound (should not exist like this)
    {
        let iter = iter.into_iter();
        let array = Array::<P>::new(domain, iter.len(), BYPASS);
        for (i, x) in iter.enumerate() {
            array.set(i, x.borrow().deref());
        }

        strat.wrap(array)
    }
}


// TODO: document static typing shenanigans (proof: we load into mono once, never unload)
unsafe impl<T: StaticallyTyped> StaticallyTyped for Array<T> {
    unsafe fn class() -> Class<'static> {
        Class::from_raw(native::mono_array_class_get(T::class().as_raw(), 1))
    }

    const IS_REFERENCE: bool = true;
}

unsafe impl<T> Referenceable for Array<T> {
    fn ptr(&self) -> *mut native::MonoObject {
        self.array as *mut _
    }
}

unsafe impl<T> Object for Array<T> {
    unsafe fn from_ptr(ptr: *mut native::MonoObject) -> Array<T> {
        Array {
            array: ptr as *mut _,
            contents: PhantomData,
        }
    }
}

impl Deref for Array<GenericObject> {
    type Target = [GenericObject];

    fn deref(&self) -> &[GenericObject] {
        unsafe {
            let ptr = native::mono_array_addr_with_size(self.array, mem::size_of::<usize>() as i32, 0);
            slice::from_raw_parts(ptr as *const GenericObject, self.len())
        }
    }
}

impl<P: StaticallyTyped> Deref for Array<P> {
    type Target = [P];

    fn deref(&self) -> &[P] {
        unsafe {
            let len = native::mono_array_length(self.array);
            let addr = native::mono_array_addr_with_size(self.array, mem::size_of::<P>() as i32, 0);
            slice::from_raw_parts(addr as *const P, len)
        }
    }
}

// TODO: move this behind an unsafe guard - it's valid in C# but UB in Rust (&mut aliasing)
impl<P: Primitive> DerefMut for Array<P> {
    fn deref_mut(&mut self) -> &mut [P] {
        unsafe {
            let len = native::mono_array_length(self.array);
            let addr = native::mono_array_addr_with_size(self.array, mem::size_of::<P>() as i32, 0);
            slice::from_raw_parts_mut(addr as *mut P, len)
        }
    }
}

impl<P: Primitive> Debug for Array<P> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str("Managed")?;
        Debug::fmt(self.deref(), fmt)
    }
}

impl<P: Primitive> Clone for Array<P> {
    fn clone(&self) -> Array<P> {
        unsafe {
            Array {
                array: native::mono_array_clone(self.array),
                contents: PhantomData,
            }
        }
    }
}




/*
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

    / *
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
     * /

    / *
    pub fn set<T: Into<ObjectReference>>(&self,
                                         index: usize,
                                         value: T) {
        assert!(index < self.len());
        unsafe {
            let ptr = native::mono_array_addr_with_size(self.0, mem::size_of::<usize>() as i32, index);
            native::mono_gc_wbarrier_set_arrayref(self.0, ptr as *mut c_void, value.into().raw());
        }
    }
     * /

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
}
*/
