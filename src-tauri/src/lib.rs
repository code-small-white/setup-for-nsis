use std::path::Path;
use std::path::PathBuf;
use std::env;
use tokio::fs;
use tokio::process::Command;

#[tauri::command]
fn get_default_install_dir() -> String {
    std::env::var("ProgramFiles(x86)")
        .unwrap_or_else(|_| String::from("C:\\Program Files (x86)"))
}

async fn gen_uninstallexe_replace_ps(install_dir:String, uninstall_exe_file_name:String,time:String) -> Result<String, String> {
    let custom_uninstall_exe: PathBuf = env::current_exe().expect("Failed to get current exe path");

    // 如果Rename-Item的目标地址已经存在,会报错,不会覆盖,不会阻碍后续行命令执行
    let content = format!(r#"
        Rename-Item '{install_dir}\{uninstall_exe_file_name}' -NewName '{install_dir}\Real {uninstall_exe_file_name}'
        Copy-Item '{custom_uninstall_exe}' '{install_dir}\{uninstall_exe_file_name}'
        "#,install_dir=install_dir,uninstall_exe_file_name=uninstall_exe_file_name,custom_uninstall_exe=custom_uninstall_exe.display());

    let mut path: PathBuf = env::temp_dir();
    path.push("setup-react-uninstallexe-replace.ps1");  

    fs::write(&path, content.as_bytes())
        .await
        .map_err(|e| format!("写入文件 {:?} 失败: {}", path, e))?;

    path.into_os_string()
        .into_string()
        .map_err(|_| "文件路径包含非 UTF‑8 字符".to_string())
}

async fn run_ps(ps: String) -> Result<i32, String> {
    println!("PowerShell command:\n{}", ps);

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let s = Command::new("powershell")
        .args(["-NoProfile","-ExecutionPolicy","Bypass","-WindowStyle","Hidden","-Command",&ps])
        .creation_flags(CREATE_NO_WINDOW)
        .status().await.map_err(|e| e.to_string())?;
    Ok(s.code().unwrap_or(-1))
}

#[tauri::command]
async fn uninstallexe_replace(install_dir:String, uninstall_exe_file_name:String)-> Result<i32, String>{
    let ps_path=gen_uninstallexe_replace_ps(install_dir, uninstall_exe_file_name).await?;
    ps_exe(r"powershell".into(),vec![ps_path]).await
}


#[tauri::command]
async fn ps_exe(file: String, args: Vec<String>) -> Result<i32, String> {

    let arglist = args
        .iter()
        .map(|a| a.replace("'", "\\\'")) // 转义引号
        .collect::<Vec<_>>()
        .join("','");
    let argfrag = if args.is_empty() { String::new() } else { format!( " -ArgumentList '{}' ", arglist ) };

    // PowerShell 提权命令：RunAs 管理员、隐藏窗口、等待退出
    let ps = format!(
        "$p = Start-Process -FilePath '{}' {} -Verb RunAs -WindowStyle Hidden -PassThru; \
         $p.WaitForExit(); exit $p.ExitCode",
        file.replace('\'', "''"),
        argfrag,
    );


    run_ps(ps).await
}

#[tauri::command]
async fn read_nsis_log() -> Result<String, String> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| String::from("C:\\Windows"));
    let path = PathBuf::from(windir).join("Temp").join("modo-nsis.log");

    fs::read_to_string(&path)
        .await
        .map_err(|e| format!("read {} failed: {}", path.display(), e))
}


#[tauri::command]
async fn check_path_exists(path: String) -> (bool, bool) {
    // 返回 (是否存在, 是否是目录)
    let p = Path::new(&path);
    (p.exists(), p.is_dir())
}

#[tauri::command]
async fn copy_file(src: String, dst: String) -> Result<i32, String> {
    let ps = format!(
                r#"$src=\"{s}\";$dst=\"{d}\";$p=Split-Path -Parent $dst; if($p -and !(Test-Path -LiteralPath $p)){{New-Item -ItemType Directory -Path $p -Force|Out-Null}}; Copy-Item -LiteralPath $src -Destination $dst -Force -Verb RunAs -WindowStyle Hidden -PassThru;"#,
                s=src, d=dst
            );
    ps_exe(r"powershell".into(),vec![ps]).await
}

#[tauri::command]
async fn rename_if_exists(src: String, dst: String) -> Result<i32, String> {
    let ps = format!(
            r#"$src=\"{s}\";$dst=\"{d}\"; if(!(Test-Path -LiteralPath $src)){{exit 10}}; $p=Split-Path -Parent $dst; if($p -and !(Test-Path -LiteralPath $p)){{New-Item -ItemType Directory -Path $p -Force|Out-Null}}; if(Test-Path -LiteralPath $dst){{Remove-Item -LiteralPath $dst -Force}}; Move-Item -LiteralPath $src -Destination $dst -Force -Verb RunAs -WindowStyle Hidden -PassThru;"#,
            s=src, d=dst
        );
    ps_exe(r"powershell".into(),vec![ps]).await
}

#[tauri::command]
fn current_exe_path() -> Result<String, String> {
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    println!("启动参数: {:?}", args);
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_default_install_dir,ps_exe,read_nsis_log,check_path_exists,copy_file,rename_if_exists,current_exe_path,uninstallexe_replace])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
