#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod emojis;

use std::sync::atomic::{AtomicIsize, Ordering};

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Stroke, Vec2};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Accessibility::SetWinEventHook;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId, IsWindow,
    SetForegroundWindow, MSG,
};

#[link(name = "user32")]
unsafe extern "system" {
    fn AttachThreadInput(idAttach: u32, idAttachTo: u32, fAttach: i32) -> i32;
}

const VK_V: u16 = 0x56;
const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;

/// Cửa sổ ngoài (Discord...) đang focus gần nhất
static LAST_HWND: AtomicIsize = AtomicIsize::new(0);
static OWN_HWND: AtomicIsize = AtomicIsize::new(0);


unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: *mut std::ffi::c_void,
    event: u32,
    hwnd: *mut std::ffi::c_void,
    id_object: i32,
    id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if event == EVENT_SYSTEM_FOREGROUND && id_object == 0 && id_child == 0 && !hwnd.is_null() {
        let own = OWN_HWND.load(Ordering::SeqCst);
        let val = hwnd as isize;
        if val != own {
            LAST_HWND.store(val, Ordering::SeqCst);
        }
    }
}

/// Dùng SetWinEventHook để lắng nghe sự kiện đổi Foreground Window (0% CPU, không polling)
fn spawn_focus_tracker(own_hwnd: isize) {
    OWN_HWND.store(own_hwnd, Ordering::SeqCst);
    // Lưu ngay foreground hiện tại
    unsafe {
        let fg = GetForegroundWindow() as isize;
        if fg != 0 && fg != own_hwnd {
            LAST_HWND.store(fg, Ordering::SeqCst);
        }
    }

    std::thread::spawn(move || unsafe {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            std::ptr::null_mut(),
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if !hook.is_null() {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                DispatchMessageW(&msg);
            }
        }
    });
}

/// Ép focus về cửa sổ cũ (vượt giới hạn SetForegroundWindow của Windows)
fn restore_focus(target: isize) {
    if target == 0 {
        return;
    }
    unsafe {
        if IsWindow(target as _) == 0 {
            return;
        }
        let fg = GetForegroundWindow();
        let fg_thread = GetWindowThreadProcessId(fg, std::ptr::null_mut());
        let my_thread = GetCurrentThreadId();

        if fg_thread != my_thread {
            AttachThreadInput(my_thread, fg_thread, 1);
            SetForegroundWindow(target as _);
            AttachThreadInput(my_thread, fg_thread, 0);
        } else {
            SetForegroundWindow(target as _);
        }
    }
}




fn key_input(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Giả lập Ctrl+V vào cửa sổ đang giữ focus (Discord)
fn send_ctrl_v() {
    let inputs = [
        key_input(VK_CONTROL, false),
        key_input(VK_V, false),
        key_input(VK_V, true),
        key_input(VK_CONTROL, true),
    ];
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Default,
    Omori2,
}

struct EmojiApp {
    current_tab: Tab,
    hovered: Option<&'static str>,
}

impl EmojiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let hwnd: isize = match cc.window_handle().map(|h| h.as_raw()) {
            Ok(RawWindowHandle::Win32(h)) => h.hwnd.get(),
            _ => 0,
        };

        spawn_focus_tracker(hwnd);

        Self {
            current_tab: Tab::Default,
            hovered: None,
        }
    }

    fn copy_emoji(&self, url: &str) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(url);
        }
        let target = LAST_HWND.load(Ordering::SeqCst);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(30));
            restore_focus(target);
            std::thread::sleep(std::time::Duration::from_millis(80));
            send_ctrl_v();
        });
    }
}

impl eframe::App for EmojiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut new_hovered: Option<&'static str> = None;

        let panel_frame = Frame::default()
            .fill(Color32::from_rgb(30, 31, 34))
            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 52, 57)))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(12));

        let mut selected_emoji: Option<&'static str> = None;

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            // Thanh tiêu đề có thể nắm kéo chuột để di chuyển cửa sổ
            ui.horizontal(|ui| {
                // Tiêu đề đóng vai trò là tay nắm kéo cửa sổ (Drag Window), không cho phép bôi đen text (không có con trỏ I-beam)
                let title_response = ui.add(
                    egui::Label::new(
                        egui::RichText::new("✨ Discord Emojis")
                            .size(15.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .selectable(false)
                    .sense(egui::Sense::drag()),
                );

                if title_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                }

                if title_response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                // Nút điều khiển cửa sổ: Thu nhỏ (-) và Đóng (X)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("X")
                                .size(13.0)
                                .strong()
                                .color(Color32::from_rgb(220, 221, 222)),
                        )
                        .fill(Color32::from_rgb(43, 45, 49))
                        .corner_radius(CornerRadius::same(6))
                        .min_size(Vec2::new(26.0, 24.0)),
                    );

                    if close_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    ui.add_space(4.0);

                    let min_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("-")
                                .size(14.0)
                                .strong()
                                .color(Color32::from_rgb(220, 221, 222)),
                        )
                        .fill(Color32::from_rgb(43, 45, 49))
                        .corner_radius(CornerRadius::same(6))
                        .min_size(Vec2::new(26.0, 24.0)),
                    );

                    if min_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });

            ui.add_space(8.0);

            // Tab Buttons: Active có nền màu tím Discord/xanh, Inactive có nền tối với text trắng sáng
            ui.horizontal(|ui| {
                let is_default = self.current_tab == Tab::Default;
                let default_text_color = if is_default {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(240, 240, 245) // Màu trắng sáng khi không active
                };
                let default_bg = if is_default {
                    Color32::from_rgb(88, 101, 242) // Tím Discord đặc trưng
                } else {
                    Color32::from_rgb(43, 45, 49)
                };

                let default_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new("General")
                            .strong()
                            .color(default_text_color),
                    )
                    .fill(default_bg)
                    .corner_radius(CornerRadius::same(6)),
                );
                if default_btn.clicked() {
                    self.current_tab = Tab::Default;
                }

                let is_omori = self.current_tab == Tab::Omori2;
                let omori_text_color = if is_omori {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(240, 240, 245) // Màu trắng sáng khi không active
                };
                let omori_bg = if is_omori {
                    Color32::from_rgb(88, 101, 242)
                } else {
                    Color32::from_rgb(43, 45, 49)
                };

                let omori_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new("Omori 2")
                            .strong()
                            .color(omori_text_color),
                    )
                    .fill(omori_bg)
                    .corner_radius(CornerRadius::same(6)),
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

            let items: Vec<&'static str> = emojis_list.iter().copied().filter(|u| !u.is_empty()).collect();
            const CELL: f32 = 44.0;
            const GAP: f32 = 8.0;

            ui.spacing_mut().item_spacing = Vec2::new(GAP, GAP);

            // Trừ chừa chỗ cho thanh cuộn (scrollbar)
            let avail = ui.available_width() - ui.spacing().scroll.bar_width - 4.0;
            let cols = (((avail + GAP) / (CELL + GAP)).floor() as usize).max(1);
            let rows = items.len().div_ceil(cols);
            let current_hovered = self.hovered;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, CELL, rows, |ui, row_range| {
                    for row in row_range {
                        ui.horizontal(|ui| {
                            for &url in items.iter().skip(row * cols).take(cols) {
                                let base = url.split('?').next().unwrap_or(url);
                                let is_hot = current_hovered == Some(url);

                                // Chỉ phát GIF động khi chuột đang hover vào chính emoji này
                                let display_url = if is_hot {
                                    if base.contains(".webp") && url.contains("animated=true") {
                                        base.replace(".webp", ".gif")
                                    } else {
                                        base.to_string()
                                    }
                                } else {
                                    base.replace(".gif", ".png")
                                };

                                let img = egui::Image::new(display_url)
                                    .fit_to_exact_size(Vec2::new(38.0, 38.0))
                                    .corner_radius(CornerRadius::same(6));

                                let response = ui.add(
                                    egui::Button::image(img)
                                        .fill(Color32::from_rgb(43, 45, 49))
                                        .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 10)))
                                        .corner_radius(CornerRadius::same(8))
                                        .min_size(Vec2::new(CELL, CELL)),
                                );

                                if response.hovered() {
                                    new_hovered = Some(url);
                                }

                                if response.clicked() {
                                    selected_emoji = Some(url);
                                }
                            }
                        });
                    }
                });
        });

        // Chỉ repaint khi trạng thái hover thay đổi hoặc đang hover xem GIF
        if self.hovered != new_hovered {
            self.hovered = new_hovered;
            ctx.request_repaint();
        }
        if self.hovered.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        if let Some(url) = selected_emoji {
            self.copy_emoji(url);
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Cấu hình Cửa sổ Picker: Borderless, Always On Top
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
        Box::new(|cc| Ok(Box::new(EmojiApp::new(cc)))),
    )
}
