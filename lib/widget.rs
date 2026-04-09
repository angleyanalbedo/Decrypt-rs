// lib/widget.rs
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(clippy::needless_update)]
#![allow(unused_assignments)]

use crate::loadicon::*;
use fltk::{prelude::*, *};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

include!(concat!(env!("OUT_DIR"), "/widget.rs"));

use crate::path::*;

#[derive(Debug, Clone)]
pub struct Widget {
    pub window: MainWindow,
    pub should_stop: bool,
    pub on_process: bool,
}

impl Widget {
    // initialize a new window and show it.
    pub fn new() -> Self {
        let mut w = Widget {
            window: Self::make_window(),
            should_stop: false,
            on_process: false,
        };
        w
    }

    fn make_window() -> MainWindow {
        let mut ui = MainWindow::make_window();
        load_icon_from_resource(ui.main_window.clone());
        //initialize MainWindow widgets
        Self::reflush_active(ui.clone());
        ui.p_bar.set_maximum(1.0);
        ui.p_bar.set_minimum(0.0);
        ui.p_bar.set_value(0.0);

        let mut ui_weak = ui.clone();
        ui.rb_deal_file.set_callback(move |_| {
            ui_weak.rb_deal_dir.set_value(!ui_weak.rb_deal_file.value());
            Self::reflush_active(ui_weak.clone());
        });

        let mut entry = ui.en_deal_file.clone();
        Self::handle_dnd_filenames(entry);

        let mut ui_weak = ui.clone();
        ui.bn_deal_file.set_callback(move |_| {
            let (i, str) = Self::choose_files();
            if i {
                ui_weak.en_deal_file.set_value(&str);
            }
        });

        let mut ui_weak = ui.clone();
        ui.rb_deal_dir.set_callback(move |_| {
            ui_weak.rb_deal_file.set_value(!ui_weak.rb_deal_dir.value());
            Self::reflush_active(ui_weak.clone());
        });

        let mut entry = ui.en_deal_dir.clone();
        Self::handle_dnd_dir(entry);

        let mut ui_weak = ui.clone();
        ui.bn_deal_dir.set_callback(move |_| {
            let (i, str) = Self::choose_dir();
            if i {
                ui_weak.en_deal_dir.set_value(&str);
            }
        });

        let mut ui_weak = ui.clone();
        ui.rb_save_orig.set_callback(move |_| {
            ui_weak
                .rb_save_other
                .set_value(!ui_weak.rb_save_orig.value());
            Self::reflush_active(ui_weak.clone());
        });

        let mut ui_weak = ui.clone();
        ui.rb_save_other.set_callback(move |_| {
            ui_weak
                .rb_save_orig
                .set_value(!ui_weak.rb_save_other.value());
            Self::reflush_active(ui_weak.clone());
        });

        let mut entry = ui.en_save_other.clone();
        Self::handle_dnd_dir(entry);

        let mut ui_weak = ui.clone();
        ui.bn_save_other.set_callback(move |_| {
            let (i, str) = Self::choose_dir();
            if i {
                ui_weak.en_save_other.set_value(&str);
            }
        });

        let mut ui_weak = ui.clone();
        ui.cb_ext_name.set_callback(move |_| {
            Self::reflush_active(ui_weak.clone());
        });

        let mut ui_weak = ui.clone();
        ui.cb_suffix_name.set_callback(move |_| {
            Self::reflush_active(ui_weak.clone());
        });

        ui.bn_link.set_callback(|_| {
            let str = include_str!("../resource/statement.txt");
            fltk::dialog::message_title("免责声明");
            // fltk::dialog::message_set_hotspot(true);
            fltk::dialog::message_icon_label("!");
            fltk::dialog::message_default(str);
        });

        ui.bn_about.set_callback(|_| {
            Self::show_about_dialog();
        });

        ui.bn_viewer.set_callback(|_| {
            Self::show_viewer_dialog();
        });

        // let ui_weak = ui.clone();
        ui.main_window.set_callback(move |w| {
            if let Event::Close = app::event() {
                w.hide();
                app::quit();
                std::process::exit(0);
            }
        });

        ui
    }

    fn show_about_dialog() {
        let mut ui = AboutDialog::make_window();

        let mut buf = fltk::text::TextBuffer::default();
        let log = include_str!("../resource/updatelog.txt");
        buf.set_text(log);
        ui.td_text.set_buffer(buf);

        let version = env!("CARGO_PKG_VERSION");
        let mut ver_str = ui.td_version.label();
        ver_str.push_str(version);
        ui.td_version.set_label(&ver_str);

        // Close this dialog
        ui.bn_about_enter.set_callback(move |_| {
            <DoubleWindow as fltk::prelude::WidgetBase>::delete(ui.about_dialog.clone());
        });
    }

    fn show_viewer_dialog() {
        let mut win = window::DoubleWindow::new(0, 0, 700, 460, "内置查看器");
        let mut en_path = input::Input::new(20, 20, 540, 30, "");
        en_path.set_tooltip("选择要查看的解密输出文件");
        let mut bn_pick = button::Button::new(570, 20, 110, 30, "选择文件...");
        let mut bn_check = button::Button::new(570, 60, 110, 30, "开始检查");
        let mut td = text::TextDisplay::new(20, 100, 660, 340, "");
        let mut buf = text::TextBuffer::default();
        td.set_buffer(buf.clone());
        td.set_text_font(enums::Font::Courier);
        win.end();
        win.make_resizable(true);
        win.show();

        let mut en_path_weak = en_path.clone();
        bn_pick.set_callback(move |_| {
            let (ok, path) = Self::choose_single_file();
            if ok {
                en_path_weak.set_value(&path);
            }
        });

        let mut en_path_weak = en_path.clone();
        let mut buf_weak = buf.clone();
        bn_check.set_callback(move |_| {
            let path = en_path_weak.value();
            if path.trim().is_empty() {
                fltk::dialog::alert_default("请先选择文件!");
                return;
            }
            let result = Self::analyze_file(Path::new(path.trim()));
            buf_weak.set_text(&result);
        });

        en_path.set_trigger(CallbackTrigger::EnterKey);
        let mut bn_check_weak = bn_check.clone();
        en_path.set_callback(move |_| {
            bn_check_weak.do_callback();
        });
    }

    fn reflush_active(mut ui: MainWindow) {
        if ui.rb_deal_file.value() {
            ui.en_deal_file.activate();
            ui.bn_deal_file.activate();
            ui.en_deal_dir.deactivate();
            ui.bn_deal_dir.deactivate();
            ui.cb_recursive.deactivate();
        } else {
            ui.en_deal_file.deactivate();
            ui.bn_deal_file.deactivate();
            ui.en_deal_dir.activate();
            ui.bn_deal_dir.activate();
            ui.cb_recursive.activate();
        }
        if ui.rb_save_orig.value() {
            ui.cb_backup.activate();
            ui.en_save_other.deactivate();
            ui.bn_save_other.deactivate();
        } else {
            ui.cb_backup.deactivate();
            ui.en_save_other.activate();
            ui.bn_save_other.activate();
        }
        if ui.cb_ext_name.value() {
            ui.en_ext_name.activate();
        } else {
            ui.en_ext_name.deactivate();
        }
        if ui.cb_suffix_name.value() {
            ui.en_suffix_name.activate();
        } else {
            ui.en_suffix_name.deactivate();
        }
    }

    fn choose_files() -> (bool, String) {
        let mut success: bool = false;
        let mut str = String::new();
        let mut fnfc =
            dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseMultiFile);
        match fnfc.try_show() {
            Ok(FileDialogAction::Success) => {
                let filenames = fnfc.filenames();
                (success, str) = vec_pathbuf_to_string(&filenames, ';');
            }
            _ => (),
        }
        (success, str)
    }

    fn choose_single_file() -> (bool, String) {
        let mut success = false;
        let mut path = String::new();
        let mut fnfc = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
        match fnfc.try_show() {
            Ok(FileDialogAction::Success) => {
                (success, path) = pathbuf_to_string(fnfc.filename());
            }
            _ => (),
        }
        (success, path)
    }

    fn analyze_file(path: &Path) -> String {
        if !path.exists() {
            return format!("文件不存在:\n{}", path.to_string_lossy());
        }
        if !path.is_file() {
            return format!("目标不是文件:\n{}", path.to_string_lossy());
        }

        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                return format!("读取文件信息失败:\n{}\n错误: {}", path.to_string_lossy(), e);
            }
        };

        let mut file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return format!("打开文件失败:\n{}\n错误: {}", path.to_string_lossy(), e);
            }
        };

        let mut sample = vec![0u8; 4096];
        let read_len = match file.read(&mut sample) {
            Ok(n) => n,
            Err(e) => {
                return format!("读取文件失败:\n{}\n错误: {}", path.to_string_lossy(), e);
            }
        };
        sample.truncate(read_len);

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let detected = Self::detect_file_type(&sample);
        let hex = Self::to_hex_preview(&sample, 96);
        let text_preview = Self::to_text_preview(&sample, 300);
        let suggest = Self::ext_suggestion(&ext, detected);

        format!(
            "文件: {}\n大小: {} bytes\n当前后缀: {}\n识别结果: {}\n\n建议: {}\n\n文件头(hex):\n{}\n\n可读文本预览:\n{}",
            path.to_string_lossy(),
            meta.len(),
            if ext.is_empty() { "(无)" } else { &ext },
            detected,
            suggest,
            hex,
            text_preview
        )
    }

    fn detect_file_type(buf: &[u8]) -> &'static str {
        if buf.starts_with(b"%PDF-") {
            "PDF 文档"
        } else if buf.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            "PNG 图片"
        } else if buf.starts_with(&[0xFF, 0xD8, 0xFF]) {
            "JPEG 图片"
        } else if buf.starts_with(b"GIF87a") || buf.starts_with(b"GIF89a") {
            "GIF 图片"
        } else if buf.starts_with(b"BM") {
            "BMP 图片"
        } else if buf.starts_with(b"PK\x03\x04") {
            "ZIP 容器（可能是 docx/xlsx/pptx/zip）"
        } else if buf.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
            "OLE 文档（可能是 doc/xls/ppt）"
        } else if buf.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
            "7z 压缩包"
        } else if buf.starts_with(b"Rar!\x1A\x07") {
            "RAR 压缩包"
        } else if buf.starts_with(&[0x1F, 0x8B]) {
            "GZIP 压缩包"
        } else if buf.starts_with(b"MZ") {
            "Windows 可执行文件"
        } else if buf.starts_with(&[0x7F, b'E', b'L', b'F']) {
            "ELF 可执行文件"
        } else if buf.is_empty() {
            "空文件"
        } else {
            "未知/二进制数据"
        }
    }

    fn ext_suggestion(ext: &str, detected: &str) -> String {
        if detected.contains("ZIP") {
            if ["docx", "xlsx", "pptx", "zip", "jar"].contains(&ext) {
                "后缀与 ZIP 容器类型基本匹配，可继续尝试打开。".to_string()
            } else {
                "文件头像 ZIP 容器，建议尝试后缀: docx / xlsx / pptx / zip。".to_string()
            }
        } else if detected.contains("PDF") {
            if ext == "pdf" {
                "后缀与 PDF 匹配。".to_string()
            } else {
                "文件头像 PDF，建议后缀改为 pdf。".to_string()
            }
        } else if detected.contains("JPEG") {
            if ["jpg", "jpeg"].contains(&ext) {
                "后缀与 JPEG 匹配。".to_string()
            } else {
                "文件头像 JPEG，建议后缀改为 jpg 或 jpeg。".to_string()
            }
        } else if detected.contains("PNG") {
            if ext == "png" {
                "后缀与 PNG 匹配。".to_string()
            } else {
                "文件头像 PNG，建议后缀改为 png。".to_string()
            }
        } else if detected.contains("GIF") {
            if ext == "gif" {
                "后缀与 GIF 匹配。".to_string()
            } else {
                "文件头像 GIF，建议后缀改为 gif。".to_string()
            }
        } else if detected.contains("OLE") {
            "文件头像旧版 Office 文档，建议尝试 doc/xls/ppt。".to_string()
        } else if detected.contains("未知") {
            "未识别出明确文件头，可换几个候选后缀再试。".to_string()
        } else {
            "文件头已识别，可按识别类型调整后缀。".to_string()
        }
    }

    fn to_hex_preview(buf: &[u8], limit: usize) -> String {
        if buf.is_empty() {
            return "(空)".to_string();
        }
        let n = buf.len().min(limit);
        let mut out = String::new();
        for (i, b) in buf[..n].iter().enumerate() {
            if i > 0 {
                if i % 16 == 0 {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
            out.push_str(&format!("{:02X}", b));
        }
        if buf.len() > limit {
            out.push_str("\n...");
        }
        out
    }

    fn to_text_preview(buf: &[u8], limit: usize) -> String {
        if buf.is_empty() {
            return "(空)".to_string();
        }
        let n = buf.len().min(limit);
        let text = String::from_utf8_lossy(&buf[..n]);
        text.chars()
            .map(|c| {
                if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                    '.'
                } else {
                    c
                }
            })
            .collect()
    }

    fn choose_dir() -> (bool, String) {
        let mut success: bool = false;
        let mut str = String::new();
        let mut fnfc = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
        match fnfc.try_show() {
            Ok(FileDialogAction::Success) => {
                let dir = fnfc.filename();
                (success, str) = pathbuf_to_string(dir);
            }
            _ => (),
        }
        (success, str)
    }

    fn handle_dnd_filenames(mut entry: fltk::input::Input) {
        entry.handle({
            let mut dnd = false;
            let mut released = false;
            let mut entry_weak = entry.clone();
            move |_, ev| match ev {
                Event::DndEnter => {
                    dnd = true;
                    true
                }
                Event::DndRelease => {
                    released = true;
                    true
                }
                Event::DndLeave => {
                    dnd = false;
                    released = false;
                    true
                }
                Event::DndDrag => true,
                Event::Paste => {
                    if dnd && released {
                        let text = app::event_text();
                        let (b1, v) = split_string_into_vec_pathbuf(text, '\n');
                        if !b1 {
                            return false;
                        }
                        match verify_path_vec(&v) {
                            PathVecType::OnlyFiles => {
                                // we use a timeout to avoid pasting the path into the input widget
                                let mut en = entry_weak.clone();
                                let mut r = false;
                                app::add_timeout3(0.01, {
                                    move |_| {
                                        let (b2, str) = vec_pathbuf_to_string(&v, ';');
                                        if !b2 {
                                            return;
                                        }
                                        en.set_value(&str);
                                        dnd = false;
                                        released = false;
                                        r = true;
                                    }
                                });
                                r
                            }
                            _ => {
                                let mut en = entry_weak.clone();
                                app::add_timeout3(0.01, {
                                    move |_| {
                                        en.set_value("");
                                    }
                                });
                                false
                            }
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            }
        });
    }

    fn handle_dnd_dir(mut entry: fltk::input::Input) {
        entry.handle({
            let mut dnd = false;
            let mut released = false;
            let mut entry_weak = entry.clone();
            move |_, ev| match ev {
                Event::DndEnter => {
                    dnd = true;
                    true
                }
                Event::DndRelease => {
                    released = true;
                    true
                }
                Event::DndLeave => {
                    dnd = false;
                    released = false;
                    true
                }
                Event::DndDrag => true,
                Event::Paste => {
                    if dnd && released {
                        let text = app::event_text();
                        let (b1, v) = split_string_into_vec_pathbuf(text, '\n');
                        if !b1 {
                            return false;
                        }
                        match verify_path_vec(&v) {
                            PathVecType::OnlyDir => {
                                // we use a timeout to avoid pasting the path into the input widget
                                let mut en = entry_weak.clone();
                                let mut r = false;
                                app::add_timeout3(0.01, {
                                    move |_| {
                                        let (b2, str) = vec_pathbuf_to_string(&v, ';');
                                        if !b2 {
                                            return;
                                        }
                                        en.set_value(&str);
                                        dnd = false;
                                        released = false;
                                        r = true;
                                    }
                                });
                                r
                            }
                            _ => {
                                let mut en = entry_weak.clone();
                                app::add_timeout3(0.01, {
                                    move |_| {
                                        en.set_value("");
                                    }
                                });
                                false
                            }
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            }
        });
    }
}
