// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs, path::PathBuf, process::Command, process, io};
use std::path::Path;

const UNINSTALL_EXE_NAME: &str="Real Uninstall.exe";



/// 写入日志到桌面的 app.log.txt 文件


/// 写入日志到桌面的 app.log.txt 文件（不返回值）
pub fn write_log(message: &str) {
    use std::fs::{OpenOptions, create_dir_all};
    use std::io::Write;
    use std::path::PathBuf;
    // 获取桌面路径（Windows / macOS / Linux）
    let desktop = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .map(|home| home.join("Desktop"));

    // 如果取不到桌面，就放到当前目录
    let log_path = match desktop {
        Ok(path) => path.join("app.log.txt"),
        Err(_) => PathBuf::from("app.log.txt"),
    };

    // 确保目录存在（一般桌面肯定有）
    if let Some(parent) = log_path.parent() {
        let _ = create_dir_all(parent);
    }

    // 打开文件（追加写入）
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(file, "{}", message);
    }

    // 不返回值，不 panic，保证调用端始终不会出错
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

/// 返回匹配的第一个文件路径；没有则返回 None
fn find_real_exe(dir: impl AsRef<Path>) -> io::Result<Option<PathBuf>> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                let lower = name.to_ascii_lowercase();
                if lower.starts_with("real") && lower.ends_with(".exe") {
                    return Ok(Some(path));
                }
            }
        }
    }
    Ok(None)
}

/// 仅判断是否存在
fn has_real_exe(dir: impl AsRef<Path>) -> io::Result<bool> {
    Ok(find_real_exe(dir)?.is_some())
}

//把启动命令
async fn handle_silent() -> io::Result<()>{
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
    write_log("静默2 return");
    write_log(&dest.to_string_lossy().into_owned());
    write_log(&args.join(" "));

    Command::new(&args[0])
        .args(&args[1..]);

    // 退出当前进程，返回新进程的退出码
    // std::process::exit(0);
    Ok(())
}



fn main() {
    if is_silent_mode() {
        write_log("静默1");
        let _ = handle_silent();
        write_log("静默23 return");
        return;
    }

    setup_react_lib::run()
}
