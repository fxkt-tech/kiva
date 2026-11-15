# Windows Blender 连接记录

- 环境：Windows 11 + WSL2；Windows Blender 5.3.0，路径 `C:\Program Files\Blender Foundation\Blender 5.3\blender.exe`。
- 用户原窗口进程42272保留；建立独立工作窗口进程66724，连接测试返回其真实PID。后续进程ID可能变化，不作为持久配置。
- 安装官方项目 `https://github.com/ahujasid/mcp-for-blender` 的 addon.py 到 Windows 用户 `AppData/Roaming/Blender Foundation/Blender/5.3/scripts/addons/blender_mcp.py`，启用并保存用户偏好。2026-11-14 下载件 SHA256：`35e5f317b932d15b485a37d914f7c9d7b527c7ea527f526569c1c111e5f39a05`。
- 现有用户窗口未能确认通过键盘自动启用插件，因此改由独立Blender工作窗口在启动脚本中启用；未覆盖或保存用户原未保存场景。
- 已用本机 `127.0.0.1:9876` 的插件协议成功执行 `get_scene_info` 与 `execute_code`，返回场景3个默认对象、Blender5.3.0与进程66724。未绑定外部网卡。
- `get_telemetry_consent` 返回false。
- Codex全局注册 `blender` MCP：`/home/asuka/.local/bin/uvx mcp-for-blender==2.2.9`，环境BLENDER_HOST=127.0.0.1、BLENDER_PORT=9876。本会话原工具列表不会因此自动变化，可通过标准MCP客户端或插件桥操作；新会话使用已注册配置。
- 独立工作文件位于项目 `art/wasteland/`；不启动游戏开发服务，不触碰模拟或旧运行资产。

## 标准MCP链路验证
使用mcp-for-blender==2.2.9的stdio服务完成ClientSession.initialize、list_tools（10项）及get_scene_info调用，服务名MCP for Blender，插件protocol16握手通过，成功返回Windows场景对象。验证客户端已退出，Blender插件监听保留。
