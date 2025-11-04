use std::env;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

static MAIN_SETUP_EXE: &[u8] = include_bytes!("../resources/setup.exe");

#[tauri::command]
fn get_default_install_dir() -> String {
    std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| String::from("C:\\Program Files (x86)"))
}

async fn gen_logfile_create_ps(log_path: &str) -> Result<String, String> {
    let content = r#"
    # 功能: 创建文件 → 隐藏 → Everyone 可读
    # ===============================

    # 目标文件路径（可修改）
    $filePath = "FILEPATH"


    "initFile=true" | Set-Content -Path $filePath -Encoding UTF8

    # 2️⃣ 设置文件为隐藏
    # 添加 Hidden 属性
    $attr = (Get-Item $filePath).Attributes
    if (-not ($attr -band [System.IO.FileAttributes]::Hidden)) {
        Set-ItemProperty -Path $filePath -Name Attributes -Value ($attr -bor [System.IO.FileAttributes]::Hidden)
    }

    # 3️⃣ 设置 Everyone 只读权限
    # 先获取现有 ACL
    $acl = Get-Acl $filePath
    $rule = New-Object System.Security.AccessControl.FileSystemAccessRule("Everyone", "Read", "Allow")
    # 拒绝写入
    $denyWriteRule = New-Object System.Security.AccessControl.FileSystemAccessRule("Everyone", "Write", "Deny")

    # 添加规则
    $acl.SetAccessRule($rule)
    #$acl.SetAccessRule($denyWriteRule)
    Set-Acl $filePath $acl

    Write-Host "✅ 文件创建并设置完成：$filePath"

        "#.replace("\"FILEPATH\"", &format!("\"{}\"", log_path));

    let mut path: PathBuf = env::temp_dir();
    path.push("logfile_create.ps1");

    fs::write(&path, content.as_bytes())
        .await
        .map_err(|e| format!("写入文件 {:?} 失败: {}", path, e))?;

    path.into_os_string()
        .into_string()
        .map_err(|_| "文件路径包含非 UTF‑8 字符".to_string())
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

async fn run_ps(ps: String) -> Result<i32, String> {
    println!("PowerShell command:\n{}", ps);

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let args = vec![
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-WindowStyle",
        "Hidden",
        "-Command",
        &ps,
    ];

    // 在开头插入 "2" 和 "3"
    let res = Command::new("powershell")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.code().unwrap_or(-1))
}

#[tauri::command]
async fn start_main_exe(app: tauri::AppHandle, path: String) {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-Command",
            &format!(r#"Start-Process -FilePath '{}' -WindowStyle Normal"#, path),
        ])
        .spawn()
        .unwrap();
    app.exit(0);
}

#[tauri::command]
async fn uninstallexe_replace(
    install_dir: String,
    uninstall_exe_file_name: String,
) -> Result<i32, String> {
    let ps_path = gen_uninstallexe_replace_ps(install_dir, uninstall_exe_file_name).await?;
    ps_exe(r"powershell".into(), vec![ps_path]).await
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
async fn ps_exe(file: String, args: Vec<String>) -> Result<i32, String> {
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
        "$p = Start-Process -FilePath '{}' {} -Verb RunAs -WindowStyle Hidden -PassThru; \
         $p.WaitForExit(); exit $p.ExitCode",
        file.replace('\'', "''"),
        argfrag
    );

    run_ps(ps).await
}

#[tauri::command]
async fn read_nsis_log(path: String) -> Result<String, String> {
    // 异步打开文件
    let mut file = match File::open(path).await {
        Ok(file) => file,
        Err(_) => return Err("文件打开失败".to_string()), // 返回错误信息
    };

    // 创建一个向量来存储文件内容
    let mut buffer = Vec::new();

    // 异步读取文件内容
    if let Err(_) = file.read_to_end(&mut buffer).await {
        return Err("文件读取失败".to_string());
    }
    // 强制将字节数据转换为 UTF-8 字符串，忽略无效字符
    let utf8_string = String::from_utf8_lossy(&buffer);

    // 返回文件内容（即使有乱码）
    Ok(utf8_string.to_string())
}

#[tauri::command]
async fn reset_file(path: String) -> Result<(), String> {
    let ps_path = gen_logfile_create_ps(&path).await?;
    let _ = ps_exe(r"powershell".into(), vec!["-File".to_string(), ps_path]).await;
    Ok(())
}

#[tauri::command]
async fn check_path_exists(path: String) -> bool {
    let p = Path::new(&path);
    p.exists()
}

#[tauri::command]
async fn path_is_dir(path: String) -> bool {
    let p = Path::new(&path);
    p.is_dir()
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

#[tauri::command]
fn uninstall_self_after_exit(app: tauri::AppHandle) -> Result<(), String> {
    let exe_path = env::current_exe().unwrap();
    let parent_pid = std::process::id();

    let script = format!(
        r#"
        $p = Get-Process -Id '{}' -ErrorAction SilentlyContinue
        if ($p) {{
            # 等待主进程退出（阻塞，不循环）
            $p.WaitForExit()
        }}
        #Start-Sleep -Seconds 1
        Remove-Item '{}' -Force
        "#,
        parent_pid,
        exe_path.display()
    );

    // 启动 PowerShell 进程来执行脚本
    Command::new("powershell")
        .arg("-Command")
        .arg(script)
        .creation_flags(0x00000200 | 0x08000000) // CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS
        .spawn()
        .expect("Failed to start the process");
    // Ok(())
    // std::process::exit(0);
    app.exit(0);
    Ok(())
    // 由调用者在外层尽快退出进程，释放句柄
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_main_exe,
            release_main_setup_exe,
            start_cmd,
            get_default_install_dir,
            ps_exe,
            read_nsis_log,
            reset_file,
            check_path_exists,
            path_is_dir,
            current_exe_path,
            uninstallexe_replace,
            uninstall_self_after_exit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
