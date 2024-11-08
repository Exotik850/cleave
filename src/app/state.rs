
use glam::DVec2;
use wgpu::core::command::Rect;
use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

use crate::selection::{
    modes::{Direction, SelectionMode},
    UserSelection,
};

#[derive(Debug, Default)]
pub struct CleaveState {
    pub selection: UserSelection,
    mouse_position: DVec2,
    mode: SelectionMode,
    size: Option<(f32, f32)>,
    mods: ModifiersState,
}

impl CleaveState {
    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(event);
            }
            WindowEvent::ModifiersChanged(mods) => self.mods = mods.state(),
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = DVec2::new(position.x, position.y);
                if let Some(drag) = self.selection.drag.as_mut() {
                    drag.w = position.x as f32 - drag.x;
                    drag.h = position.y as f32 - drag.y;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => self.start_drag(),
                (ElementState::Released, MouseButton::Left) => self.end_drag(),
                (_, MouseButton::Right) => self.cancel_drag(),
                _ => {}
            },
            _ => {}
        }
        // println!("Pressed: {:?}, mods: {:?}", self.pressed, self.mods);
    }

    pub fn handle_key(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        match (event.state, code) {
            (ElementState::Pressed, KeyCode::ArrowUp) => {
                self.handle_move(Direction::Up);
            }
            (ElementState::Pressed, KeyCode::ArrowDown) => {
                self.handle_move(Direction::Down);
            }
            (ElementState::Pressed, KeyCode::ArrowLeft) => {
                self.handle_move(Direction::Left);
            }
            (ElementState::Pressed, KeyCode::ArrowRight) => {
                self.handle_move(Direction::Right);
            }
            (ElementState::Pressed, KeyCode::ShiftLeft) => {
                self.set_mode(SelectionMode::InverseResize);
            }
            (ElementState::Released, KeyCode::ShiftLeft | KeyCode::ControlLeft) => {
                self.set_mode(SelectionMode::Move);
            }
            (ElementState::Pressed, KeyCode::ControlLeft) => {
                self.set_mode(SelectionMode::Resize);
            }
            _ => {}
        };
    }

    pub fn start_drag(&mut self) {
        if let Some(drag) = self.selection.drag.as_mut() {
            if drag.x != 0. && drag.y != 0. {
                return;
            }
        };
        let mouse_pos = self.mouse_position.as_vec2();
        self.selection.drag = Some(Rect {
            x: mouse_pos.x,
            y: mouse_pos.y,
            w: 0.0,
            h: 0.0,
        });
    }

    pub fn end_drag(&mut self) {
        self.selection.selection = self.selection.drag.take();
    }

    pub fn cancel_drag(&mut self) {
        self.selection.drag = None;
        self.selection.selection = None;
    }

    pub fn handle_move(&mut self, dir: Direction) -> Option<()> {
        let (width, height) = self.size?;
        let selection = self.selection.selection.as_mut()?;

        let (dx, dy) = match dir {
            Direction::Up => (0.0, -1.0),
            Direction::Down => (0.0, 1.0),
            Direction::Left => (-1.0, 0.0),
            Direction::Right => (1.0, 0.0),
        };
        let (x_delta, y_delta) = match self.mode {
            SelectionMode::Move => (dx, dy),
            SelectionMode::InverseResize => (dx, dy),
            SelectionMode::Resize => (0.0, 0.0),
        };

        if matches!(
            self.mode,
            SelectionMode::Move | SelectionMode::InverseResize
        ) {
            selection.x = (selection.x + x_delta).clamp(0.0, width);
            selection.y = (selection.y + y_delta).clamp(0.0, height);
        }

        if matches!(self.mode, SelectionMode::Move | SelectionMode::Resize) {
            selection.w = (selection.w + dx).clamp(0.0, width);
            selection.h = (selection.h + dy).clamp(0.0, height);
        }

        Some(())
    }

    pub fn size(&mut self, size: (f32, f32)) -> &mut Self {
        self.size = Some(size);
        self
    }

    pub fn set_mode(&mut self, mode: SelectionMode) -> &mut Self {
        self.mode = mode;
        self
    }
}
