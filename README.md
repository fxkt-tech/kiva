# Kiva · AI 狼人杀观战台

使用 Rust / Bevy 0.20.1 的九人 AI 狼人杀观战游戏。支持桌面与 WebAssembly / WebGL2，推荐在 WSL2 中开发、使用 Windows Chrome 或 Edge 调试。

## WSL2 准备

在 WSL 的 Linux 文件系统中放置项目（例如 `~/projects/kiva`），安装 Linux 版 Rust 与工具，避免使用 Windows 的 `cargo.exe`：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source "$HOME/.cargo/env"
rustup update stable
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

Bevy 0.20.1 至少需要 Rust 1.97.1。网页编译不需要 WSLg、ALSA 或 Linux 图形驱动；桌面运行仍需相应系统依赖。Linux 构建需要基础 C 工具链（包括 WASM 构建过程中在主机上运行的构建脚本）：

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config
```

## 网页开发

```bash
./scripts/web-dev.sh
```

在 **Windows 浏览器** 中访问 <http://localhost:8080>。脚本同时启动 Trunk（编译、监听改动、自动刷新）和本地 LLM 代理（8001 端口），退出时清理代理进程。首次编译会下载 Rust 依赖，耗时较长。网页采用现有桌面布局，小屏幕可滚动查看；当前主要面向桌面浏览器。

不配置 LLM 时，保留原有规则 AI，可直接观战。需要真实模型时，在启动前设置环境变量：

```bash
export KIVA_LLM_BASE_URL='https://your-provider.example/v1'
export KIVA_LLM_API_KEY='your-api-key'
export KIVA_LLM_MODEL='your-model'
./scripts/web-dev.sh
```

这些值仅由 Python 代理读取，浏览器通过 `/api/chat/completions` 请求，密钥不编译进 WASM、不存入浏览器。修改变量后重启脚本。代理仅监听本机，不提供公网部署所需的用户认证或配额管理。

## 网页构建与预览

```bash
trunk build --cargo-profile wasm-release
python3 scripts/web_server.py --directory dist --port 8080
```

浏览器访问 <http://localhost:8080>。`dist/` 包含 WASM、JavaScript 与 `assets/`，可作为静态网站发布；纯静态托管时没有 `/api/config`，游戏使用规则 AI。在线 LLM 模式需要部署同源 API 服务并配置服务端凭据。本地 Python 服务用于开发和预览。

当前复用的中文字体约 54 MB，会影响首次加载；正式发布时可换用精简字体，并启用服务器压缩。

## 桌面开发与验证

```bash
cargo run
# 桌面动态链接与资源热重载（仅用于本机开发）
cargo run --features desktop-dev
cargo test --lib
cargo check --target wasm32-unknown-unknown
python3 -m unittest discover -s scripts -p 'test_*.py'
```

Bevy 默认使用 WebGL2；本项目未启用 WebGPU。WSL 中的网页渲染由 Windows 浏览器执行。
