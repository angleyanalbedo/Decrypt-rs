// lib/launcher.rs
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(clippy::needless_update)]
#![allow(unused_assignments)]

use crate::loadicon::*;
use crate::theme::*;
use fltk::{prelude::*, *};
use std::env;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

include!(concat!(env!("OUT_DIR"), "/widget.rs"));

pub fn launcher_app() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    using_theme();
    make_window();
    app.run().unwrap();
}

fn make_window() {
    let mut ui = LauncherWindow::make_window();
    load_icon_from_resource(ui.launcher_win.clone());

    // 自动填充当前程序路径为默认值
    if let Ok(current_exe) = env::current_exe() {
        ui.en_exe_path.set_value(&current_exe.to_string_lossy());
    }

    // 浏览按钮回调
    let mut ui_clone = ui.clone();
    ui.bn_browse.set_callback(move |_| {
        if let Some(path) = dialog::file_chooser(
            "选择EXE程序",
            "EXE文件\t*.exe\n所有文件\t*",
            ".",
            false,
        ) {
            ui_clone.en_exe_path.set_value(&path);
        }
    });

    // 启动按钮回调
    ui.bn_start.set_callback(move |_| {
        let exe_path = ui.en_exe_path.value();
        if exe_path.is_empty() {
            dialog::alert_default("请先选择程序路径!");
            return;
        }

        let path = PathBuf::from(&exe_path);
        if !path.exists() {
            dialog::alert_default("选择的文件不存在!");
            return;
        }

        ui.launcher_win.hide();

        println!("启动程序: {}", exe_path);
        #[cfg(windows)]
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        #[cfg(windows)]
        let status = Command::new(&path)
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        #[cfg(target_family = "unix")]
        let status = Command::new(&path).status();

        match status {
            Ok(_) => {},
            Err(e) => {
                eprintln!("启动失败: {}", e);
                dialog::alert_default(&format!("启动失败: {}", e));
            }
        }

        std::process::exit(0);
    });
}
