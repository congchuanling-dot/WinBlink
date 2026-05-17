mod common;
mod hotkey;
mod ui;

use anyhow::Result;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

fn main() -> Result<()> {
    println!("WinBlink 启动成功");

    let hotkey_mgr = hotkey::HotkeyManager::new()?;
    let event_loop = EventLoop::new()?;

    let mut app = App {
        hotkey_mgr,
        window: None,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}

struct App {
    hotkey_mgr: hotkey::HotkeyManager,
    window: Option<ui::AppWindow>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            match ui::AppWindow::new(event_loop) {
                Ok(w) => self.window = Some(w),
                Err(e) => {
                    eprintln!("创建窗口失败: {e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key,
                        ..
                    },
                ..
            } => {
                if logical_key == Key::Named(NamedKey::Escape) {
                    if let Some(ref window) = self.window {
                        window.hide();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);

        if self.hotkey_mgr.poll() {
            println!("[热键] 切换窗口");
            if let Some(ref window) = self.window {
                window.toggle();
            }
        }
    }
}
