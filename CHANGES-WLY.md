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
- 安装脚本及程序内更新仍使用上游地址；更新可能恢复原版界面。本次未修改更新机制，也未发布 WLY 专用 EXE Release。
- 上游原 README 与动图保留作来源资料；本页和首页 WLY 预览说明本修改版的差异。

## 验证

2026-10-23，本机已完成正式编译、69 项测试（0 失败、0 忽略），系统代理实测可成功读取额度。后续界面尺寸、分隔线和间距微调经过格式检查、重新编译、4 项显示相关测试及示例预览检查。跨屏切换、拖动和右键未完成本次实机回归，不据此声明这些流程全部验证通过。

## 致谢与许可证

- 上游项目：[upstream-ray/codex-usage-monitor](https://github.com/upstream-ray/codex-usage-monitor)。
- 原项目 MIT 版权与许可文本完整保留于 `LICENSE`，图标声明保留于 `src/icons/providers/`。
- Adobe Source Han Sans 字体的版权、保留字体名称及 SIL Open Font License 完整保留于 `assets/fonts/LICENSE.txt`。
