# WLY 修改说明

基于 upstream-ray/codex-usage-monitor 1.10.0 的 Windows 修改版，保留上游 Git 历史及 MIT 许可证。

## 界面与交互

- 左右两组青绿色圆环，分别显示短周期和周额度，配闪电、日历图标；上方为剩余百分比，下方为本地重置时间或日期。
- 不显示中间 Codex 图标、5h / 7d 和“重置”字样，保留透明任务栏合成。
- 内置 Source Han Sans HW SC Bold，采用 3 倍尺寸绘制后缩小，改善文字和曲线边缘。
- 圆环直径 27 个逻辑像素，双组总宽 202 个逻辑像素；分隔线在可见内容空隙中居中，文字轻微上移以增加底部留白。
- 扩大组件拖动范围；调整右键菜单处理、重复启动恢复显示、启动空文本与跨屏透明绘制处理；登录检测放到后台。

## 系统代理

额度查询每次建立请求客户端时读取 Windows 当前启用的静态系统代理；支持统一地址及 `https=host:port` 格式，不固定代理端口。

显式代理环境变量优先级保持与 ureq 一致：ALL_PROXY、HTTPS_PROXY、HTTP_PROXY（兼容小写）。没有可用环境变量代理时，采用 Windows 系统代理；系统代理关闭则由原请求路径处理。无需为本版专门开启 TUN，代理服务本身仍须正常工作。PAC 自动配置脚本尚未实现。

## 构建

需要 Windows、Rust MSVC 工具链、Visual Studio C++ Build Tools 和 Windows SDK。

运行 `build.cmd`，或依次执行：

```powershell
cargo fmt --check
cargo test
cargo build --release
```

输出文件为 `target\release\codex-usage.exe`。本仓库包含构建源码、字体及许可，不包含账号凭据、个人设置、日志或本机构建缓存。

## 数据状态与更新限制

- 星号 `*` 表示缓存数据未能按时刷新，可能由连接失败等原因引起；获取新数据成功后自动清除。
- 系统代理支持不能保证代理服务或远端接口始终可达，网络失败时保留缓存并继续重试。
- 安装脚本及程序内更新已指向本 WLY 仓库。
- 上游原 README 与动图保留作来源资料；本页和首页 WLY 预览说明本修改版的差异。

## 验证

2026-10-23，本机已完成正式编译、69 项测试（0 失败、0 忽略），系统代理实测可成功读取额度。后续界面尺寸、分隔线和间距微调经过格式检查、重新编译、4 项显示相关测试及示例预览检查。跨屏切换、拖动和右键未完成本次实机回归，不据此声明这些流程全部验证通过。

## 致谢与许可证

- 上游项目：[upstream-ray/codex-usage-monitor](https://github.com/upstream-ray/codex-usage-monitor)。
- 原项目 MIT 版权与许可文本完整保留于 `LICENSE`，图标声明保留于 `src/icons/providers/`。
- Adobe Source Han Sans 字体的版权、保留字体名称及 SIL Open Font License 完整保留于 `assets/fonts/LICENSE.txt`。

## 1.10.1：固定位置与自动发布

- “显示器”选择已移到右键主菜单。
- 右键主菜单新增“固定当前位置”：记录所选显示器和逻辑坐标，重启恢复并禁止拖动，托盘宽度改变时不漂移。取消勾选可移动；手动重置、切换显示器或固定最左侧会解除当前固定。设置保存在各台电脑自己的用户目录，不打包进 EXE。
- 本地 71 项测试通过；隔离设置下连续两次实际启动恢复相同位置。跨屏和缩放完整实机回归仍未完成。
- 每次推送 main（或手动运行 Release 工作流）会先检查格式和测试，再编译 Windows x64 EXE，附 SHA256 和代码、字体许可证发布到 Releases。标签带运行编号，保留每次发布所对应的源码提交。
- 更新功能按 Cargo.toml 的主版本、次版本、修订号判断新版本；后续功能更新时应递增版本，提交 Cargo.lock，测试后同步 main。程序及安装脚本的来源为本 WLY 仓库。
- 下载入口：[最新版 codex-usage.exe](https://github.com/wuliya66666/Codex-usage-monitor/releases/latest/download/codex-usage.exe)。其他 Windows 10/11 x64 电脑下载后可直接运行，仍需本机 Codex 登录和可用网络。EXE 未进行数字签名，不附带任何账号凭据。
