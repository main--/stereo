use super::object::GenericObject;
use safety::GcPtrStrategy;

#[derive(Debug)]
pub enum MonoValue<S: GcPtrStrategy<GenericObject>> {
    I32(i32),
    ObjectRef(Option<S::Target>),
}
