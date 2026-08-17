#!/usr/bin/env node
/**
 * KnowteQuiz 首次安装引导脚本
 * 检查环境 → 安装依赖 → 构建 → 选择启动模式
 *
 * 用法: node setup.cjs
 */

const { spawn, execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const readline = require('readline');

const isWindows = process.platform === 'win32';
const projectRoot = __dirname;
const tauriDir = path.join(projectRoot, 'src-tauri');

// ── 工具函数 ──────────────────────────────────────────────

function log(msg) {
  console.log(`[KnowteQuiz] ${msg}`);
}

function logOk(msg) {
  console.log(`[KnowteQuiz] ✅ ${msg}`);
}

function logWarn(msg) {
  console.log(`[KnowteQuiz] ⚠️  ${msg}`);
}

function logErr(msg) {
  console.log(`[KnowteQuiz] ❌ ${msg}`);
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

function checkCommand(cmd) {
  try {
    execSync(`${cmd} --version`, { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

function getNodeVersion() {
  try {
    return execSync('node --version', { encoding: 'utf-8' }).trim();
  } catch {
    return null;
  }
}

function getRustVersion() {
  try {
    return execSync('cargo --version', { encoding: 'utf-8' }).trim();
  } catch {
    return null;
  }
}

function cargoEnv() {
  return isWindows
    ? { Path: `${process.env.Path};${process.env.USERPROFILE}\\.cargo\\bin` }
    : { PATH: `${process.env.PATH}:${process.env.HOME}/.cargo/bin` };
}

// ── 交互菜单 ──────────────────────────────────────────────

function askChoice(question, options) {
  return new Promise((resolve) => {
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
    console.log();
    console.log(question);
    options.forEach((opt, i) => {
      console.log(`  ${i + 1}) ${opt.label}`);
      if (opt.desc) console.log(`     ${opt.desc}`);
    });
    console.log();
    rl.question('  请选择 [1]: ', (answer) => {
      rl.close();
      const idx = parseInt(answer, 10) - 1;
      if (idx >= 0 && idx < options.length) {
        resolve(options[idx].value);
      } else {
        resolve(options[0].value); // 默认第一个
      }
    });
  });
}

// ── 主流程 ──────────────────────────────────────────────

async function main() {
  console.log();
  console.log('╔══════════════════════════════════════╗');
  console.log('║     KnowteQuiz 首次安装引导         ║');
  console.log('╚══════════════════════════════════════╝');
  console.log();

  // ── Step 1: 检查环境 ──
  log('步骤 1/4: 检查运行环境...');

  const nodeVer = getNodeVersion();
  if (!nodeVer) {
    logErr('未找到 Node.js。请安装 Node.js 18+：https://nodejs.org');
    process.exit(1);
  }
  logOk(`Node.js ${nodeVer}`);

  const rustVer = getRustVersion();
  if (!rustVer) {
    logWarn('未找到 Rust。桌面模式需要 Rust，Web 模式不需要。');
  } else {
    logOk(rustVer);
  }

  // ── Step 2: 安装依赖 ──
  log('步骤 2/4: 安装 npm 依赖...');
  if (!fs.existsSync(path.join(projectRoot, 'node_modules'))) {
    await runCommand('npm', ['install']);
    logOk('npm 依赖安装完成');
  } else {
    logOk('node_modules 已存在，跳过安装');
  }

  // ── Step 3: 构建前端 ──
  log('步骤 3/4: 构建前端...');
  const distExists = fs.existsSync(path.join(projectRoot, 'dist', 'index.html'));
  if (!distExists) {
    await runCommand('npm', ['run', 'build']);
    logOk('前端构建完成');
  } else {
    logOk('dist/ 已存在，跳过构建');
  }

  // ── Step 4: 选择启动模式 ──
  log('步骤 4/4: 选择启动模式...');

  const modes = [
    {
      label: 'Web 模式（推荐，无需 Rust）',
      desc: '启动 Axum 服务器，浏览器访问 http://localhost:14200',
      value: 'web',
    },
    {
      label: '开发模式（需要 Rust + Tauri CLI）',
      desc: '热重载桌面应用，修改代码后自动刷新',
      value: 'dev',
    },
    {
      label: '桌面 + Web 同时启动（需要 Rust）',
      desc: '同时运行 Tauri 桌面窗口和 Web 服务器',
      value: 'both',
    },
    {
      label: '仅构建（不启动）',
      desc: '构建桌面应用 release 版本',
      value: 'build',
    },
    {
      label: '退出',
      desc: '不启动任何服务',
      value: 'quit',
    },
  ];

  // 如果没有 Rust，隐藏需要 Rust 的选项
  const availableModes = rustVer ? modes : modes.filter((m) => !['dev', 'both', 'build'].includes(m.value));

  const choice = await askChoice('你想以哪种模式启动？', availableModes);

  console.log();
  switch (choice) {
    case 'web':
      log('🌐 启动 Web 服务...');
      await runCommand('cargo', ['run', '--', '--mode=web'], { cwd: tauriDir, env: cargoEnv() });
      break;
    case 'dev':
      log('🚀 启动开发模式...');
      await runCommand('npm', ['run', 'tauri:dev'], { env: cargoEnv() });
      break;
    case 'both':
      log('🖥️🌐 启动桌面 + Web...');
      await runCommand('cargo', ['run', '--', '--mode=both'], { cwd: tauriDir, env: cargoEnv() });
      break;
    case 'build':
      log('🔧 构建桌面应用...');
      await runCommand('npm', ['run', 'tauri:build'], { env: cargoEnv() });
      logOk('构建完成！运行 node start.cjs 启动应用');
      break;
    case 'quit':
      logOk('安装完成！下次运行：');
      console.log('  node start.cjs          # 启动桌面应用');
      console.log('  node start.cjs --web    # 启动 Web 服务');
      console.log('  node start.cjs --dev    # 启动开发模式');
      break;
  }
}

main().catch((err) => {
  logErr(err.message);
  process.exit(1);
});
