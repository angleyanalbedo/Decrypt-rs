// lib/launcher.rs
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(clippy::needless_update)]
#![allow(unused_assignments)]

use crate::loadicon::*;
use crate::pe_modify::{modify_pe_version_info, read_pe_version_info};
use crate::theme::*;
use fltk::{prelude::*, *};
use std::env;
use std::fs;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
        let mut chooser = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseFile);
        chooser.set_title("选择EXE程序");
        chooser.set_filter("EXE文件\t*.{exe,EXE}\n所有文件\t*");
        let _ = chooser.set_directory(&PathBuf::from("."));
        chooser.show();

        let selected = chooser.filename();
        if !selected.as_os_str().is_empty() {
            ui_clone
                .en_exe_path
                .set_value(selected.to_string_lossy().as_ref());
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

        let launcher_exe = match env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                dialog::alert_default(&format!("获取当前程序路径失败: {}", e));
                return;
            }
        };
        let decrypt_path = launcher_exe.with_file_name("decrypt.exe");
        if !decrypt_path.exists() {
            dialog::alert_default(&format!(
                "未找到 decrypt.exe: {}",
                decrypt_path.to_string_lossy()
            ));
            return;
        }

        let metadata = match read_pe_version_info(&path, None) {
            Ok(m) => m,
            Err(e) => {
                dialog::alert_default(&format!("读取目标程序PE元数据失败: {}", e));
                return;
            }
        };

        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let temp_dir = env::temp_dir().join(format!(
            "decrypt-launcher-{}-{}",
            std::process::id(),
            millis
        ));
        if let Err(e) = fs::create_dir_all(&temp_dir) {
            dialog::alert_default(&format!("创建临时目录失败: {}", e));
            return;
        }
        let temp_decrypt_path = temp_dir.join("decrypt.exe");
        if let Err(e) = fs::copy(&decrypt_path, &temp_decrypt_path) {
            dialog::alert_default(&format!("复制 decrypt.exe 到临时目录失败: {}", e));
            return;
        }
        #[cfg(windows)]
        {
            let decrypt_dll =
                launcher_exe.with_file_name(format!("{}.dll", env!("CARGO_PKG_NAME")));
            if !decrypt_dll.exists() {
                dialog::alert_default(&format!(
                    "未找到 decrypt.dll: {}",
                    decrypt_dll.to_string_lossy()
                ));
                return;
            }
            let temp_decrypt_dll = temp_dir.join(format!("{}.dll", env!("CARGO_PKG_NAME")));
            if let Err(e) = fs::copy(&decrypt_dll, &temp_decrypt_dll) {
                dialog::alert_default(&format!("复制 decrypt.dll 到临时目录失败: {}", e));
                return;
            }
        }

        if let Err(e) = modify_pe_version_info(&path, &temp_decrypt_path, &metadata) {
            dialog::alert_default(&format!("伪装临时 decrypt.exe 失败: {}", e));
            return;
        }

        ui.launcher_win.hide();

        println!(
            "启动伪装后的临时 decrypt.exe: {}",
            temp_decrypt_path.to_string_lossy()
        );
        #[cfg(windows)]
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        #[cfg(windows)]
        let status = Command::new(&temp_decrypt_path)
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        #[cfg(target_family = "unix")]
        let status = Command::new(&temp_decrypt_path).status();

        match status {
            Ok(_) => {}
            Err(e) => {
                eprintln!("启动失败: {}", e);
                dialog::alert_default(&format!("启动失败: {}", e));
            }
        }
        if let Err(e) = fs::remove_dir_all(&temp_dir) {
            eprintln!("清理临时目录失败: {}", e);
        }

        std::process::exit(0);
    });
}
