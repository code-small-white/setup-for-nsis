#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
import { execSync } from 'child_process';

// 1. 获取当前工作目录
const currentDir = process.cwd();
const pkgPath = path.join(currentDir, 'package.json');

// 2. 检查当前目录是否有 package.json
if (!fs.existsSync(pkgPath)) {
  console.error(`❌ 错误: 当前目录[${currentDir}]下未找到 package.json。`);
  console.error('请在项目根目录（包含 package.json 的目录）中运行此命令。');
  process.exit(1);
}

// 获取命令行参数（跳过前两个默认参数：node 和脚本路径）
const inputPath = process.argv[2];

if (!inputPath) {
  console.error('请提供一个文件路径作为参数。');
  console.log('用法: node copy-setup.js <文件路径>');
  process.exit(1);
}

// 转为绝对路径（相对于当前工作目录）
const absolutePath = path.resolve(inputPath);

// 检查路径是否存在且是文件
if (!fs.existsSync(absolutePath)) {
  console.error(`错误: 文件不存在 - ${absolutePath}`);
  process.exit(1);
}

if (!fs.statSync(absolutePath).isFile()) {
  console.error(`错误: 路径不是一个文件 - ${absolutePath}`);
  process.exit(1);
}

// 目标路径
const destDir = path.join(process.cwd(), 'src-tauri', 'resources');
const destPath = path.join(destDir, 'setup.exe');

// 确保目标目录存在
if (!fs.existsSync(destDir)) {
  fs.mkdirSync(destDir, { recursive: true });
}

// 复制文件
try {
  fs.copyFileSync(absolutePath, destPath);
  console.log(`✅ 成功复制文件到: ${destPath}`);
} catch (err) {
  console.error('❌ 复制文件失败:', err.message);
  process.exit(1);
}

// 执行 npm run build
try {
  console.log('🚀 正在执行 npm run build ...');
  execSync('npm run gen-exe', { stdio: 'inherit' });
  console.log('🎉 构建完成！');
} catch (err) {
  console.error('❌ 构建失败:', err.message);
  process.exit(1);
}
