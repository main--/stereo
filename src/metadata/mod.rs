// pub mod assembly; // ???
mod image;
mod class;
mod method;
pub use self::image::Image;
pub use self::class::Class;
pub use self::method::Method;

pub struct TypeToken(pub u32);
pub struct MethodToken(pub u32);
