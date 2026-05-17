mod common;
mod hotkey;
mod system;
mod ui;

use anyhow::Result;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

fn main() -> Result<()> {
    println!("WinBlink 启动成功");

    // Load installed apps
    print!("正在索引应用程序... ");
    match system::get_installed_apps() {
        Ok(apps) => {
            println!("找到 {} 个应用", apps.len());
            let limit = apps.len().min(10);
            for app in apps.iter().take(limit) {
                println!("  - {}", app.name);
            }
            if apps.len() > 10 {
                println!("  ... 还有 {} 个应用", apps.len() - 10);
            }
        }
        Err(e) => {
            eprintln!("索引应用失败: {e}");
        }
    }

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
                Ok(mut w) => {
                    // Load apps and pass them to the window
                    match system::get_installed_apps() {
                        Ok(apps) => {
                            println!("[UI] 索引到 {} 个应用", apps.len());
                            w.set_apps(apps);
                        }
                        Err(e) => eprintln!("[UI] 索引应用失败: {e}"),
                    }
                    self.window = Some(w);
                }
                Err(e) => {
                    eprintln!("创建窗口失败: {e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // Forward to UI for egui handling
        if let Some(ref mut window) = self.window {
            window.handle_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(ref mut window) = self.window {
                    window.render();
                }
            }
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

        // Request redraw when visible
        if let Some(ref window) = self.window {
            if window.is_visible() {
                self.window.as_ref().unwrap().window.request_redraw();
            }
        }
    }
}
