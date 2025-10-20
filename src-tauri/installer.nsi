SilentInstall silent
AutoCloseWindow true
ShowInstDetails nevershow
ShowUninstDetails nevershow
RequestExecutionLevel user

!macro NSIS_HOOK_POSTINSTALL
  ; 安装完成后自动启动主程序
  ; $INSTDIR 由 Tauri 默认安装脚本设置为最终安装目录
  ; ${MAINBINARYNAME} 由 Tauri 传入（主 exe 名，无扩展名）
  Exec '"$INSTDIR\${MAINBINARYNAME}.exe"'
!macroend