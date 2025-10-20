; ===== 变量 =====
Var LOG_FILE
Var LOG_HANDLE
Var LOG_IS_OPEN

; ===== 日志宏：初始化(清空)、继续写、结束(关闭) =====
; 初始化：清空并打开（保持打开）
!macro LOG_BEGIN
  StrCpy $LOG_FILE "$WINDIR\Temp\modo-nsis.log"
  FileOpen $LOG_HANDLE "$LOG_FILE" w
  StrCpy $LOG_IS_OPEN 1
!macroend

; 写一行：若没打开则以追加方式打开，不关闭
!macro LOG_WRITE PHASE MESSAGE
  StrCmp $LOG_IS_OPEN 1 +3
    FileOpen $LOG_HANDLE "$LOG_FILE" a
    StrCpy $LOG_IS_OPEN 1
  FileWrite $LOG_HANDLE "${PHASE}=${MESSAGE}$\r$\n"
!macroend

; 结束：显式关闭（不再写时调用）
!macro LOG_END
  StrCmp $LOG_IS_OPEN 1 0 +2
    FileClose $LOG_HANDLE
  StrCpy $LOG_IS_OPEN 0
!macroend


; ========== 你的钩子，改为使用以上日志宏 ==========

;!macro customHeader
;  CRCCheck off
;!macroend

!macro customInit
  ; 初始化时清空并打开
  !insertmacro LOG_BEGIN
  !insertmacro LOG_WRITE "init" "初始化安装器..."
!macroend

!macro customInstall

  !insertmacro LOG_WRITE "installStart" "true"
!macroend


; 安装成功：写完后关闭
Function .onInstSuccess
  !insertmacro LOG_WRITE "installFinish" "true"
  !insertmacro LOG_WRITE "mainExe" "${PRODUCT_FILENAME}.exe"
  !insertmacro LOG_WRITE "uninstallExe" "Uninstall ${PRODUCT_FILENAME}.exe"
  !insertmacro LOG_END
FunctionEnd

; 可选：安装失败时也关闭
Function .onInstFailed
  !insertmacro LOG_WRITE "installFailed" "true"
  !insertmacro LOG_END
FunctionEnd

!macro customUnInit
  !insertmacro LOG_BEGIN
  !insertmacro LOG_WRITE "uninstallStart" "true"
!macroend


; 卸载流程：单独的日志文件
!macro customUnInstall
  ; 初始化卸载日志（清空并打开）
  !insertmacro LOG_BEGIN
  !insertmacro LOG_WRITE "uninstalling" "true"
!macroend

