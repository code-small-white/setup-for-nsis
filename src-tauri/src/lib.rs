use std::env;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

static MAIN_SETUP_EXE: &[u8] = include_bytes!("../../nsis-demo/dist/minimal-repro Setup 1.0.0.exe");

#[tauri::command]
fn get_default_install_dir() -> String {
    std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| String::from("C:\\Program Files (x86)"))
}

async fn gen_uninstallexe_replace_ps(
    install_dir: String,
    uninstall_exe_file_name: String,
) -> Result<String, String> {
    let custom_uninstall_exe: PathBuf = env::current_exe().expect("Failed to get current exe path");

    // 如果Rename-Item的目标地址已经存在,会报错,不会覆盖,不会阻碍后续行命令执行
    // TODO真实卸载文件定义一个常量
    let content = format!(
        r#"
        Rename-Item '{install_dir}\{uninstall_exe_file_name}' -NewName '{install_dir}\Real Uninstall.exe'
        Copy-Item '{custom_uninstall_exe}' '{install_dir}\{uninstall_exe_file_name}'
        "#,
        install_dir = install_dir,
        uninstall_exe_file_name = uninstall_exe_file_name,
        custom_uninstall_exe = custom_uninstall_exe.display()
    );

    let mut path: PathBuf = env::temp_dir();
    path.push("setup-react-uninstallexe-replace.ps1");

    fs::write(&path, content.as_bytes())
        .await
        .map_err(|e| format!("写入文件 {:?} 失败: {}", path, e))?;

    path.into_os_string()
        .into_string()
        .map_err(|_| "文件路径包含非 UTF‑8 字符".to_string())
}

async fn run_ps(ps: String, hidden: bool) -> Result<i32, String> {
    println!("PowerShell command:\n{}", ps);

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut args = vec!["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps];

    // 在开头插入 "2" 和 "3"
    let mut cmd = Command::new("powershell");
    if hidden {
        cmd.creation_flags(CREATE_NO_WINDOW);
        args.splice(0..0, ["-WindowStyle", "Hidden"].iter().cloned());
    }
    cmd.args(args);
    let s = cmd.status().await.map_err(|e| e.to_string())?;
    Ok(s.code().unwrap_or(-1))
}

#[tauri::command]
async fn uninstallexe_replace(
    install_dir: String,
    uninstall_exe_file_name: String,
) -> Result<i32, String> {
    let ps_path = gen_uninstallexe_replace_ps(install_dir, uninstall_exe_file_name).await?;
    ps_exe(r"powershell".into(), true, vec![ps_path]).await
}

#[tauri::command]
async fn release_main_setup_exe() -> Result<PathBuf, String> {
    // TODO 生成时间错的随机文件名，避免冲突
    let exe_path = std::env::temp_dir().join("a.exe");

    // 2️⃣ 创建并写入文件
    let mut file = File::create(&exe_path)
        .await
        .map_err(|e| format!("create {} failed: {}", exe_path.display(), e))?;

    file.write_all(MAIN_SETUP_EXE)
        .await
        .map_err(|e| format!("write {} failed: {}", exe_path.display(), e))?;

    // 3️⃣ 确保写入磁盘
    file.flush()
        .await
        .map_err(|e| format!("flush {} failed: {}", exe_path.display(), e))?;

    // 4️⃣ 文件自动关闭 (file drop)

    println!("✅ 已释放到临时目录: {}", exe_path.display());
    Ok(exe_path)
}

#[tauri::command]
async fn ps_exe(file: String, hidden: bool, args: Vec<String>) -> Result<i32, String> {
    let arglist = args
        .iter()
        .map(|a| a.replace("'", "\\\'")) // 转义引号
        .collect::<Vec<_>>()
        .join("','");
    let argfrag = if args.is_empty() {
        String::new()
    } else {
        format!(" -ArgumentList '{}' ", arglist)
    };

    // PowerShell 提权命令：RunAs 管理员、隐藏窗口、等待退出
    let ps = format!(
        "$p = Start-Process -FilePath '{}' {} -Verb RunAs {} -PassThru; \
         $p.WaitForExit(); exit $p.ExitCode",
        file.replace('\'', "''"),
        argfrag,
        if hidden { "-WindowStyle Hidden" } else { "" }
    );

    run_ps(ps, hidden).await
}

fn get_nsis_log_path() -> PathBuf {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| String::from("C:\\Windows"));
    PathBuf::from(windir).join("Temp").join("modo-nsis.log")
}

#[tauri::command]
async fn read_nsis_log() -> Result<String, String> {
    let path = get_nsis_log_path();
    fs::read_to_string(&path)
        .await
        .map_err(|e| format!("read {} failed: {}", path.display(), e))
}

#[tauri::command]
async fn delete_nsis_log() -> Result<String, String> {
    let path = get_nsis_log_path();

    match fs::remove_file(&path).await {
        Ok(_) => Ok(format!("{} deleted", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(format!("{} not found (already deleted)", path.display()))
        }
        Err(e) => Err(format!("delete {} failed: {}", path.display(), e)),
    }
}

#[tauri::command]
async fn check_path_exists(path: String) -> (bool, bool) {
    // 返回 (是否存在, 是否是目录)
    let p = Path::new(&path);
    (p.exists(), p.is_dir())
}

#[tauri::command]
fn current_exe_path() -> Result<String, String> {
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn start_cmd() -> String {
    std::env::args().collect::<Vec<_>>().join(" ")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            release_main_setup_exe,
            start_cmd,
            get_default_install_dir,
            ps_exe,
            read_nsis_log,
            delete_nsis_log,
            check_path_exists,
            current_exe_path,
            uninstallexe_replace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
