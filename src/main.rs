#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod emojis;

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Stroke, Vec2};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsWindowVisible, SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE, SW_SHOW,
};

fn win_show(hwnd: isize) {
    if hwnd == 0 {
        return;
    }
    unsafe {
        ShowWindow(hwnd as _, SW_SHOW);
        ShowWindow(hwnd as _, SW_RESTORE);
        SetForegroundWindow(hwnd as _);
    }
}

fn win_hide(hwnd: isize) {
    if hwnd == 0 {
        return;
    }
    unsafe {
        ShowWindow(hwnd as _, SW_HIDE);
    }
}

fn win_is_visible(hwnd: isize) -> bool {
    if hwnd == 0 {
        return false;
    }
    unsafe { IsWindowVisible(hwnd as _) != 0 }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Default,
    Omori2,
}

struct EmojiApp {
    current_tab: Tab,
    hwnd: isize,
    _tray_icon: TrayIcon,
}

impl EmojiApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        tray_icon: TrayIcon,
        quit_id: tray_icon::menu::MenuId,
        open_id: tray_icon::menu::MenuId,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let hwnd: isize = match cc.window_handle().map(|h| h.as_raw()) {
            Ok(RawWindowHandle::Win32(h)) => h.hwnd.get(),
            _ => 0,
        };

        // Click trái vào tray -> toggle
        {
            let ctx = cc.egui_ctx.clone();
            TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    if win_is_visible(hwnd) {
                        win_hide(hwnd);
                    } else {
                        win_show(hwnd);
                        ctx.request_repaint();
                    }
                }
            }));
        }

        // Menu chuột phải
        {
            let ctx = cc.egui_ctx.clone();
            MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
                if event.id == quit_id {
                    std::process::exit(0);
                } else if event.id == open_id {
                    win_show(hwnd);
                    ctx.request_repaint();
                }
            }));
        }

        Self {
            current_tab: Tab::Default,
            hwnd,
            _tray_icon: tray_icon,
        }
    }

    fn hide(&self) {
        win_hide(self.hwnd);
    }

    fn copy_emoji(&self, url: &str) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(url);
        }
        self.hide();
    }
}

impl eframe::App for EmojiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let panel_frame = Frame::default()
            .fill(Color32::from_rgb(30, 31, 34))
            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 52, 57)))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(12));

        let mut selected_emoji: Option<&str> = None;
        let mut close = false;

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            // Thanh tiêu đề + Nút đóng
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("✨ Discord Emojis")
                        .size(16.0)
                        .strong()
                        .color(Color32::WHITE),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("✕").size(14.0).color(Color32::GRAY)).clicked() {
                        close = true;
                    }
                });
            });

            ui.add_space(8.0);

            // Tab Buttons
            ui.horizontal(|ui| {
                let default_btn = ui.selectable_label(
                    self.current_tab == Tab::Default,
                    egui::RichText::new("General").strong(),
                );
                if default_btn.clicked() {
                    self.current_tab = Tab::Default;
                }

                let omori_btn = ui.selectable_label(
                    self.current_tab == Tab::Omori2,
                    egui::RichText::new("Omori 2").strong(),
                );
                if omori_btn.clicked() {
                    self.current_tab = Tab::Omori2;
                }
            });

            ui.separator();
            ui.add_space(4.0);

            // Danh sách emoji theo tab
            let emojis_list = match self.current_tab {
                Tab::Default => emojis::DEFAULT_EMOJIS,
                Tab::Omori2 => emojis::OMORI2_EMOJIS,
            };

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

                        for &url in emojis_list {
                            if url.is_empty() {
                                continue;
                            }

                            let img = egui::Image::new(url)
                                .fit_to_exact_size(Vec2::new(38.0, 38.0))
                                .corner_radius(CornerRadius::same(6));

                            let response = ui.add(
                                egui::Button::image(img)
                                    .fill(Color32::from_rgb(43, 45, 49))
                                    .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 10)))
                                    .corner_radius(CornerRadius::same(8))
                                    .min_size(Vec2::new(44.0, 44.0)),
                            );

                            if response.hovered() {
                                response.clone().highlight();
                            }

                            if response.clicked() {
                                selected_emoji = Some(url);
                            }
                        }
                    });
                });
        });

        if let Some(url) = selected_emoji {
            self.copy_emoji(url);
        }
        if close {
            self.hide();
        }
    }
}

/// Tạo icon mặc định trong bộ nhớ cho khay hệ thống (Tray Icon)
fn create_default_tray_icon() -> tray_icon::Icon {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let dx = (x as i32) - 16;
            let dy = (y as i32) - 16;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= 14 * 14 {
                // Màu tím Discord đặc trưng (#5865F2)
                rgba.extend_from_slice(&[88, 101, 242, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    tray_icon::Icon::from_rgba(rgba, width, height).unwrap()
}

fn main() -> Result<(), eframe::Error> {
    // 1. Tạo Context Menu cho Khay hệ thống
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Thoát", true, None);
    let open_item = MenuItem::new("Mở bảng Emoji", true, None);
    let _ = tray_menu.append(&open_item);
    let _ = tray_menu.append(&quit_item);

    let quit_id = quit_item.id().clone();
    let open_id = open_item.id().clone();

    // 2. Tạo Tray Icon
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Discord Emoji Picker (Rust Native)")
        .with_icon(create_default_tray_icon())
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false)
        .build()
        .expect("Không thể tạo Tray Icon");

    // 3. Cấu hình Cửa sổ Picker: Borderless, Always On Top
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Discord Emoji Picker")
            .with_decorations(false)
            .with_always_on_top()
            .with_inner_size([440.0, 360.0])
            .with_min_inner_size([300.0, 200.0])
            .with_visible(true),
        ..Default::default()
    };

    eframe::run_native(
        "discord_emoji_picker",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(EmojiApp::new(cc, tray_icon, quit_id, open_id)))
        }),
    )
}
