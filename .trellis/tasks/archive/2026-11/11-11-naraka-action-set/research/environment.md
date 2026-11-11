# 研究环境与范围

日期：2026-11-11。仓库：/home/asuka/go/src/github.com/fxkt-tech/kiva，main基线81d9366588cce444380ecc4f6990b4cfe9683408。开始前无未提交产品改动。

客户端为用户指定的C盘Steam安装，WSL路径/mnt/c/Program Files (x86)/Steam/steamapps/common/NARAKA BLADEPOINT。源安装只读；没有运行/注入游戏，没有启动常驻开发服务。当前工作仅写Trellis规划/研究与外部研究缓存，产品src/web/assets/Cargo未修改。

复用的外部研究根目录/home/asuka/kiva-asset-research/naraka保留既有NEW2/SparseBundle/DBBL/AppRes工具，UnityPy1.25.4环境在该目录.venv。本轮源对象以原TypeTree/serialized2019.4.41f2读取，完整性范围由每个probe报告声明：稀疏读只声明所读压缩块/原对象SHA，不冒称整个大包SHA。

## 原生转换工具可用性

- 本次检查的PATH没有Unity/UnityHub/dotnet/mono/wine；ffmpeg可用，WindowsPowerShell可调用。
- /opt为空，C盘Program Files/Program Files (x86)/Unity及Unity Hub、C盘根Unity/UnityHub/Unity Hub均未发现。/mnt只存在C盘及WSL挂载。
- 只读查询Windows HKLM/HKCU卸载注册表的DisplayName/InstallLocation/DisplayVersion，Unity子串仅匹配到Visual Studio Community，没有Unity Editor/Hub条目。
- 这些是已检查位置的事实，不代表已穷尽任何手工解压/便携安装。未安装、激活或假定具有Unity原生求值环境。Humanoid原生对照仍是实施前置技术依赖。

## 研究产物与验证说明

父任务research保存精简报告、定向证据和可重跑脚本；大型配置payload/debug在研究完成后由主会话移入项目外专用缓存，归档清单记录原相对名、SHA和位置。本文不把脚本语法通过或数学spike当作完整动作接入/浏览器验收。

父/四子任务的manifest验证只核文档上下文路径有效。没有产品变更，因此本次尚未运行make check/Trunk；未来实现按implement.md执行。
