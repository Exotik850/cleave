use wgpu::core::command::Rect;

pub mod modes;
#[derive(Debug, Clone, Default)]
pub struct UserSelection {
    pub drag: Option<Rect<f32>>,
    pub selection: Option<Rect<f32>>,
}
