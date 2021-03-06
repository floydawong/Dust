pub mod buffer;
pub mod program;
pub mod rendertarget;
pub(crate) mod shader;
pub mod state;
pub mod texture;
pub mod types;
pub mod camera;

pub use buffer::*;
pub use program::*;
pub use rendertarget::*;
pub use state::*;
pub use texture::*;
pub use types::*;
pub use camera::*;

pub type Gl = std::rc::Rc<gl::Gl>;