安装pnpm 
https://pnpm.io/zh/installation
安装rust环境
<!-- 官网链接 -->
https://rust-lang.org/tools/install/
安装的时候如果遇到这个问题,可以直接选择1
```bash

Rust Visual C++ prerequisites

Rust requires a linker and Windows API libraries but they don't seem to be
available.

These components can be acquired through a Visual Studio installer.

1) Quick install via the Visual Studio Community installer
   (free for individuals, academic uses, and open source).

2) Manually install the prerequisites
   (for enterprise and advanced users).

3) Don't install the prerequisites
   (if you're targeting the GNU ABI).
```
下载demo
<!-- https://github.com/mx369/setup-for-nsis -->

进入 nsis-demo目录
npm i
如果安装electron很慢,可以使用命令 npx i-electron

- npm run build
如果执行很慢,可以 设置一下环境electron-builder的依赖下载环境变量
参照:https://www.npmjs.com/package/i-electron  的tips

返回根目录
npm i

cd src-tauri
`cargo build`

使用下面命令生成安装包
- node scripts\genExe.cjs "nsis-demo\dist\minimal-repro Setup 1.0.0.exe"
