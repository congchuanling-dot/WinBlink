use std::cell::Cell;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::Arc;

use anyhow::Result;
use egui::{Context, ScrollArea, TextEdit};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::ContextAttributesBuilder;
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::*;
use glutin::surface::SurfaceAttributesBuilder;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowLevel};

use crate::common::types::SearchItem;

pub struct AppWindow {
    pub window: Window,
    visible: Cell<bool>,
    gl: Option<GlState>,
    egui_ctx: Context,
    egui_winit: egui_winit::State,
    apps: Vec<SearchItem>,
    search_query: String,
}

struct GlState {
    _display: Display,
    _context: glutin::context::PossiblyCurrentContext,
    _surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    painter: egui_glow::Painter,
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

        let egui_ctx = Context::default();

        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let gl = GlState::new(&window).ok();

        Ok(Self {
            window,
            visible: Cell::new(false),
            gl,
            egui_ctx,
            egui_winit,
            apps: Vec::new(),
            search_query: String::new(),
        })
    }

    pub fn set_apps(&mut self, apps: Vec<SearchItem>) {
        self.apps = apps;
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

    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_winit
            .on_window_event(&self.window, event)
            .consumed
    }

    pub fn render(&mut self) {
        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.build_ui(ctx);
        });
        self.egui_winit
            .handle_platform_output(&self.window, full_output.platform_output);

        if let Some(ref gl) = self.gl {
            let paint_jobs = self
                .egui_ctx
                .tessellate(full_output.shapes, full_output.pixels_per_point);
            gl.painter.paint_and_update_textures(
                None,
                full_output.pixels_per_point,
                &paint_jobs,
                &full_output.textures_delta,
            );
        }
    }

    fn build_ui(&mut self, ctx: &Context) {
        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(28, 28, 32))
            .inner_margin(egui::Margin::same(8.0));
        let search_filter: &str = &self.search_query;

        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                ui.add_space(4.0);

                let search_response = ui.add(
                    TextEdit::singleline(&mut self.search_query)
                        .hint_text("搜索应用...")
                        .desired_width(f32::INFINITY)
                        .text_color(egui::Color32::WHITE)
                        .font(egui::FontId::proportional(16.0)),
                );

                if self.visible.get() && !search_response.has_focus() {
                    search_response.request_focus();
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                let search_lower = search_filter.to_lowercase();
                let filtered: Vec<&SearchItem> = self
                    .apps
                    .iter()
                    .filter(|a| {
                        search_filter.is_empty()
                            || a.name.to_lowercase().contains(&search_lower)
                    })
                    .collect();

                ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        for item in filtered {
                            let label = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&item.name)
                                        .size(14.0)
                                        .color(egui::Color32::from_rgb(220, 220, 224)),
                                )
                                .sense(egui::Sense::click()),
                            );

                            if label.clicked() {
                                let _ = std::process::Command::new(&item.path).spawn();
                                self.hide();
                            }
                        }
                    });
            });
    }
}

impl GlState {
    fn new(window: &Window) -> Result<Self> {
        let raw_display = window.display_handle()?.as_raw();
        let raw_window = window.window_handle()?.as_raw();

        let display = unsafe { Display::new(raw_display, DisplayApiPreference::Wgl(None)) }
            .map_err(|e| anyhow::anyhow!("创建 GL Display 失败: {e}"))?;

        let config_template = ConfigTemplateBuilder::new().build();
        let config = unsafe { display.find_configs(config_template) }
            .map_err(|e| anyhow::anyhow!("查找 GL 配置失败: {e}"))?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("无可用 GL 配置"))?;

        let ctx_attrs = ContextAttributesBuilder::new().build(Some(raw_window));
        let context = unsafe { display.create_context(&config, &ctx_attrs) }
            .map_err(|e| anyhow::anyhow!("创建 GL 上下文失败: {e}"))?;

        let size = window.inner_size();
        let surface_attrs =
            SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
                raw_window,
                NonZeroU32::new(size.width).unwrap_or(NonZeroU32::MIN),
                NonZeroU32::new(size.height).unwrap_or(NonZeroU32::MIN),
            );

        let surface = unsafe { display.create_window_surface(&config, &surface_attrs) }
            .map_err(|e| anyhow::anyhow!("创建 GL 表面失败: {e}"))?;

        let context = context
            .make_current(&surface)
            .map_err(|e| anyhow::anyhow!("激活 GL 上下文失败: {e}"))?;

        let gl = unsafe {
            glow::Context::from_loader_function(|name| {
                let cname = CString::new(name).unwrap();
                display.get_proc_address(&cname) as *const _
            })
        };

        let painter = egui_glow::Painter::new(Arc::new(gl), "", None, true)
            .map_err(|e| anyhow::anyhow!("创建 Painter 失败: {e}"))?;

        Ok(Self {
            _display: display,
            _context: context,
            _surface: surface,
            painter,
        })
    }
}
