#!/usr/bin/env node
/**
 * KnowteQuiz 跨平台启动脚本
 * 支持 Windows & macOS & Linux
 * 用法: node start.cjs [选项]
 *   --dev, -d    启动开发模式（npm run tauri:dev），支持热重载
 *   --build, -b  仅构建生产版本
 *   --web, -w    启动 Web 服务模式（Axum，端口 14200）
 *   --both       同时启动桌面 + Web 服务
 *   默认: 启动生产构建（release），如未构建则自动构建
 */

const { spawn, execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const isWindows = process.platform === 'win32';
const isMac = process.platform === 'darwin';
const isLinux = process.platform === 'linux';

const projectRoot = __dirname;
const tauriDir = path.join(projectRoot, 'src-tauri');

// 根据平台确定可执行文件路径
function getExePath(profile) {
  const profileDir = path.join(tauriDir, 'target', profile);
  if (isWindows) {
    return path.join(profileDir, 'knowtequiz.exe');
  }
  if (isMac) {
    // macOS .app bundle 或纯二进制
    const appBundle = path.join(profileDir, 'bundle', 'macos', 'KnowteQuiz.app');
    if (fs.existsSync(appBundle)) {
      return path.join(appBundle, 'Contents', 'MacOS', 'knowtequiz');
    }
    return path.join(profileDir, 'knowtequiz');
  }
  // Linux
  return path.join(profileDir, 'knowtequiz');
}

function log(msg) {
  console.log(`[KnowteQuiz] ${msg}`);
}

function runCommand(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, {
      stdio: 'inherit',
      shell: true,
      cwd: opts.cwd || projectRoot,
      env: { ...process.env, ...opts.env },
    });
    child.on('close', (code) => {
      if (code !== 0) {
        reject(new Error(`Command exited with code ${code}`));
      } else {
        resolve(code);
      }
    });
  });
}

function checkCargo() {
  try {
    execSync('cargo --version', { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

function checkNpm() {
  try {
    execSync('npm --version', { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

async function buildRelease() {
  log('🔧 正在构建生产版本...');
  if (!checkNpm()) {
    console.error('❌ 未找到 npm，请先安装 Node.js');
    process.exit(1);
  }
  if (!checkCargo()) {
    console.error('❌ 未找到 Cargo，请先安装 Rust (https://rustup.rs)');
    process.exit(1);
  }

  const env = isWindows
    ? { ...process.env, Path: `${process.env.Path};${process.env.USERPROFILE}\\.cargo\\bin` }
    : { ...process.env, PATH: `${process.env.PATH}:${process.env.HOME}/.cargo/bin` };

  await runCommand('npm', ['run', 'tauri:build'], { env });
  log('✅ 构建完成');
}

async function startDev() {
  log('🚀 启动开发模式（热重载）...');
  if (!checkNpm()) {
    console.error('❌ 未找到 npm，请先安装 Node.js');
    process.exit(1);
  }

  const env = isWindows
    ? { ...process.env, Path: `${process.env.Path};${process.env.USERPROFILE}\\.cargo\\bin` }
    : { ...process.env, PATH: `${process.env.PATH}:${process.env.HOME}/.cargo/bin` };

  await runCommand('npm', ['run', 'tauri:dev'], { env });
}

async function startRelease(extraArgs = []) {
  let exePath = getExePath('release');

  // 如果 release 不存在，尝试 debug
  if (!fs.existsSync(exePath)) {
    log('⚠️ 未找到生产构建，尝试查找开发构建...');
    exePath = getExePath('debug');
  }

  // 如果 debug 也不存在，自动构建
  if (!fs.existsSync(exePath)) {
    log('⚠️ 未找到任何构建产物，开始自动构建...');
    await buildRelease();
    exePath = getExePath('release');
  }

  if (!fs.existsSync(exePath)) {
    console.error('❌ 构建后仍未找到可执行文件，请检查构建日志');
    process.exit(1);
  }

  log(`🚀 启动应用: ${exePath} ${extraArgs.join(' ')}`);
  const child = spawn(exePath, extraArgs, {
    stdio: 'inherit',
    detached: false,
    cwd: projectRoot,
  });

  child.on('error', (err) => {
    console.error('❌ 启动失败:', err.message);
    process.exit(1);
  });

  return child;
}

async function startWeb() {
  log('🌐 启动 Web 服务模式（端口 14200）...');
  if (!checkCargo()) {
    console.error('❌ 未找到 Cargo，请先安装 Rust (https://rustup.rs)');
    process.exit(1);
  }

  const env = isWindows
    ? { ...process.env, Path: `${process.env.Path};${process.env.USERPROFILE}\\.cargo\\bin` }
    : { ...process.env, PATH: `${process.env.PATH}:${process.env.HOME}/.cargo/bin` };

  await runCommand('cargo', ['run', '--', '--mode=web'], { cwd: tauriDir, env });
}

async function startBoth() {
  log('🖥️🌐 同时启动桌面 + Web 服务...');
  if (!checkCargo()) {
    console.error('❌ 未找到 Cargo，请先安装 Rust (https://rustup.rs)');
    process.exit(1);
  }

  const env = isWindows
    ? { ...process.env, Path: `${process.env.Path};${process.env.USERPROFILE}\\.cargo\\bin` }
    : { ...process.env, PATH: `${process.env.PATH}:${process.env.HOME}/.cargo/bin` };

  await runCommand('cargo', ['run', '--', '--mode=both'], { cwd: tauriDir, env });
}

async function main() {
  const args = process.argv.slice(2);
  const isDev = args.includes('--dev') || args.includes('-d');
  const shouldBuild = args.includes('--build') || args.includes('-b');
  const isWeb = args.includes('--web') || args.includes('-w');
  const isBoth = args.includes('--both');
  const extraArgs = args.filter((a) => !['--dev', '-d', '--build', '-b', '--web', '-w', '--both'].includes(a));

  log(`检测平台: ${process.platform} (${isWindows ? 'Windows' : isMac ? 'macOS' : 'Linux'})`);

  if (shouldBuild) {
    await buildRelease();
    return;
  }

  if (isWeb) {
    await startWeb();
  } else if (isBoth) {
    await startBoth();
  } else if (isDev) {
    await startDev();
  } else {
    await startRelease(extraArgs);
  }
}

main().catch((err) => {
  console.error('❌ 错误:', err.message);
  process.exit(1);
});
