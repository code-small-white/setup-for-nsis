实现原理:
给已有的nsis setup包;再套一个tauri exe壳子;
壳子主要提供UI界面,实际的安装/卸载,通过已有的nsis setup包完成;
额外需要做的工作: 在你的electron-builder配置加入nsis-demo\installer.nsh
实现流程:


可能的失败情况:
{ code: 32, kind: Uncategorized, message: "另一个程序正在使用此文件，进程无法访问。" }
{ code: 2, kind: NotFound, message: "系统找不到指定的文件。" }