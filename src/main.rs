#![windows_subsystem = "windows"]

mod app;
mod args;
mod context;


fn main() -> anyhow::Result<()> {
    let mut app = app::App::new();
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}
