# QuackStore (嘎嘎密库) 🦆

> **三合一极简套件**：高通硬件级 RKP 签发 + 多后端 Keystore 管理器 + 全自动定时同步

QuackStore 是专为 Android Root 用户打造的硬件级密钥认证与防检测全自动化套件。它剥离了冗余的离线提取工具，专注于**“合法硬件证书签发、主流 Keystore 统一管理、全周期静默维保”**。

## 🌟 核心特性

1. **硬件 TA 驱动的 RKP 远程密钥签发**
   - 直接调用高通芯片的 KeyMint TA 硬件生成 CSR（支持 V2、V1 AIDL、V1 Native 协议自动回退）；
   - 自动向 Google 服务器申领合法的硬件级 ECDSA P-256 证书链，直过 Play Integrity 强校验。
2. **多后端 Keystore 统一纳管**
   - 自动识别并自适应管理当前激活的底层 Keystore 模块：**OhMyKeymint**、**Tricky Store**、**TEESimulator**；
   - 自动同步至各自的真实 Keybox 路径（`/data/misc/keystore/omk`、`/data/adb/tricky_store`、`/data/adb/teesim`）。
3. **Rust 原生后台调度守护进程 (quackd daemon)**
   - 纯 Rust 异步常驻，开机自启，默认每周四 16:00 自动触发硬件签发与同步；
   - 支持防遗漏补偿（手机休眠唤醒后自动补跑）与网络退避重试。
4. **纯硬件 EC 证书链，不掺软件密钥**
   - keybox 只包含芯片 TA 签发、Google RKP 认证的 ECDSA 链；
   - 不再注入 AOSP 软件 RSA 占位块——那份私钥来自公开的源码，一旦并入会让整份 keybox 被判定为软件签名，设备级完整性直接退化到 BASIC；
   - OhMyKeymint 在缺少 RSA 节点时会自行回退到内置的 AOSP 条目，三个后端均可正常加载纯 EC keybox。
5. **高可用双通道网络容灾 (Smart Failover)**
   - 默认优先直连 Google 官方端点；
   - 若特定网络环境出现连接超时或解析阻断，系统底层自动无缝切换至高可用备用反代通道重试，保障全天候签发成功率。
6. **双核心一体化极简 WebUI**
   - 告别繁琐多层级菜单，重塑为「自动签名与调度」和「密钥与目标管理」双主控面板。


## 📦 架构说明

```txt
crates/
  quack-core/          核心抽象: Sysroot, AppPaths, 统一 JSON Envelope 契约
  quack-platform/      系统交互: resetprop, pm, 模块与网络底层封装
  quack-rkp/           RKP 协议核心: DICE/CBOR/COSE 密码学处理与网络交互
  quack-rkp-device/    高通硬件 KeyMint TA 通信与签名
  quack-tricky-store/  Keystore 管理器: 目标应用列表、策略与多后端适配器
  quackd/              主二进制与 Rust 调度守护进程 (quackd daemon)
module/                Root 模块部署包: 启动脚本、环境预检与 WebUI 资源
ui/                    Vue 3 + Tailwind CSS 4 双核心极简前端
```

## 📄 开源许可证与仓库

* **GitHub 仓库**：[https://github.com/long45343/QuackStore](https://github.com/long45343/QuackStore)
* **开源协议**：本项目基于 MIT License 开源，衍生并精简自 Duck-ToolBox，保留原作者及上游组件版权。详见 [LICENSE](LICENSE)。
