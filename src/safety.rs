use managed::{Referenceable, Object, GcHandle};

pub unsafe trait GcPtrStrategy<T: Object> {
    type Target: Referenceable;

    fn wrap(&self, t: T) -> Self::Target;
}

pub(crate) const BYPASS: &'static StackRefs = &StackRefs(());

#[derive(Debug)]
pub struct StackRefs(());

// TODO: /thoroughly/ document all of this

impl StackRefs {
    pub unsafe fn i_promise_to_never_store_references_anywhere_other_than_the_stack() -> StackRefs {
        StackRefs(())
    }
}

unsafe impl<T: Object> GcPtrStrategy<T> for StackRefs {
    type Target = T;

    fn wrap(&self, t: T) -> T { t }
}


#[derive(Debug)]
pub struct GcHandles;

unsafe impl<T: Object> GcPtrStrategy<T> for GcHandles {
    type Target = GcHandle<T>;

    fn wrap(&self, t: T) -> GcHandle<T> {
        GcHandle::new(t)
    }
}
