// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs, process::Command, process, io};
use std::path::Path;
use std::os::windows::process::CommandExt;

const UNINSTALL_EXE_NAME: &str="Real Uninstall.exe";

pub fn write_log(message: &str) {
    println!("{}",&message);
}


fn is_silent_mode() -> bool {
    std::env::args()
        .skip(1)
        .any(|a| a.eq_ignore_ascii_case("/S") || a.eq_ignore_ascii_case("--silent"))
}

fn parse_install_dir(args: Vec<String>) -> String {
    args.join(" ")                 
        .split("_?=")              
        .nth(1)                    
        .map(|s| s.trim().to_string()) 
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            eprintln!("缺少 NSIS 参数：_?=安装目录");
            process::exit(1);
        })
}

//把真实卸载可执行文件复制到当前目录；把启动命令透传到真实卸载可执行文件上
fn handle_silent() -> io::Result<()>{
    write_log("handle_silent");

    let current_exe = env::current_exe()?;
    let exe_dir = current_exe.parent().unwrap();
    let dest = exe_dir.join(UNINSTALL_EXE_NAME);
    
    let install_dir = parse_install_dir(std::env::args().collect());

    let src = Path::new(&install_dir).join(UNINSTALL_EXE_NAME);
    write_log(&format!("uninstall.exe 路径: {}", src.display()));
    fs::copy(&src, &dest)?;
    // 获取当前命令行参数（Vec<String>）
    let mut args: Vec<String> = env::args().collect();

    // 替换第一个元素为新 exe 的路径
    args[0] = dest.to_string_lossy().into_owned();

    // 执行新 exe，继承当前控制台
    write_log(&dest.to_string_lossy().into_owned());
    write_log(&args.join(" "));

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    Command::new(&args[0])
        .args(&args[1..])      
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    // std::process::exit(0);
    Ok(())
}

fn hide_console_window() {
    // 直接使用 WinAPI（无第三方依赖）
    type HWND = *mut core::ffi::c_void;

    extern "system" {
        fn GetConsoleWindow() -> HWND;
        fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
        // fn FreeConsole() -> i32;
    }

    const SW_HIDE: i32 = 0;

    unsafe {
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            // 先隐藏窗口，再释放控制台
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}



fn main() {
    if is_silent_mode() {
        // hide_console_window();
        write_log("静默启动");
        let _ = handle_silent();
        return;
    }

    setup_react_lib::run()
}
