use std::cell::Cell;

use anyhow::Result;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize, Size},
    event_loop::ActiveEventLoop,
    window::{Window, WindowLevel},
};

pub struct AppWindow {
    pub window: Window,
    visible: Cell<bool>,
}

impl AppWindow {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let attrs = Window::default_attributes()
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_inner_size(Size::Physical(PhysicalSize::new(650, 420)))
            .with_visible(false);

        let window = event_loop.create_window(attrs)?;

        if let Some(monitor) = window.primary_monitor() {
            let ms = monitor.size();
            let ws = window.inner_size();
            let x = (ms.width as i32 - ws.width as i32).max(0) / 2;
            let y = (ms.height as i32 - ws.height as i32).max(0) / 2;
            let _ = window.set_outer_position(PhysicalPosition::new(x, y));
        }

        Ok(Self {
            window,
            visible: Cell::new(false),
        })
    }

    pub fn toggle(&self) {
        if self.visible.get() {
            self.window.set_visible(false);
            self.visible.set(false);
            println!("[窗口] 隐藏");
        } else {
            self.window.set_visible(true);
            self.window.focus_window();
            self.visible.set(true);
            println!("[窗口] 显示");
        }
    }

    pub fn hide(&self) {
        if self.visible.get() {
            self.window.set_visible(false);
            self.visible.set(false);
            println!("[窗口] 隐藏 (ESC)");
        }
    }
}
