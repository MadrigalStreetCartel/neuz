<div align=right>
  <a href="README.md">English</a> | <a href="README.de.md">Deutsch</a>
</div>

&nbsp;&nbsp;&nbsp;&nbsp;

![banner]

- [下载](#下载)
- [社区](#社区)
- [使用方法](#使用方法)
  - [战斗模式](#战斗模式)
  - [刷怪自动化](#刷怪自动化)
    - [使用要求](#使用要求)
    - [快捷栏配置](#快捷栏配置)
  - [辅助自动化](#辅助自动化)
  - [自动喊话](#自动喊话-autoshout)
- [常见问题](#常见问题)

> `Neuz` 是一个增强版的 Flyff Universe, 飞飞, 新飞飞, 网页飞飞 客户端 & **智能机器人软件**，通过自定义图像识别，实现各种自动化操作 (例如 自动打怪脚本)。

> [!WARNING]  
> 本页面为**自动翻译**。英文版本为**唯一权威版本**，本翻译可能存在错误。

# 下载
[![Build release](https://github.com/MadrigalStreetCartel/neuz/actions/workflows/main.yml/badge.svg)](https://github.com/MadrigalStreetCartel/neuz/actions/workflows/main.yml)
- 最新版本: [下载][download]
- 旧版本 (仅限 Windows): [版本存档](./releases)

查看 [更新日志][changelog]！

# 社区
**v0.12.1 是由最初开发团队维护并发布的最后一个版本，后续版本完全由社区开发。**

当前计划的功能：
- **自动检测并避开 BOSS（巨人）**
- **支持自定义脚本（基于 DSL 语言）**，可用于编写个性化的移动和战斗逻辑
- **全面支持 Mac & Linux 平台**
- **通过 Docker 容器进行自动化部署，实现并行运行**

加入我们的 Discord 社群: https://discord.gg/WR6FuNEYj6

# 开发指南

## 依赖环境：
- 安装 `nodejs` 最新版本（推荐最新版）
- 安装 `yarn` (https://classic.yarnpkg.com/en/docs/install)
- 安装 `rustup` (https://rust-lang.org/tools/install)
- 使用 rustup 安装最新稳定版 Rust: `rustup install stable`

## 编译与构建：
1. 在项目根目录创建 `build` 文件夹
2. 在根目录运行 `yarn` 以安装依赖
3. 运行 `yarn tauri dev` 启动开发模式
4. 运行 `yarn tauri build` 进行生产环境构建

## 提交代码前：
- 在 `src-tauri` 目录运行 `cargo clippy` 检查代码质量
- 在 `src-tauri` 目录运行 `cargo fmt` 进行代码格式化

# 使用方法

1. **以管理员身份**运行 Neuz
2. 选择或创建一个角色配置
3. 点击 `Play`
4. 选择战斗风格
5. 根据个人需求调整设置
6. 按下 `ENGAGE` 开始运行

## 战斗模式

- 机器人会在 `ENGAGE` 后自动运行。
- 按 `DISENGAGE` 可完全停止自动化操作。
- **从 0.15.0 版本起支持后台运行**，可在后台执行任务。

## 刷怪自动化

当你需要 **升级角色、刷装备、任务道具、赚取 Penya** 时，可以使用刷怪自动化功能。

推荐在 **怪物密集的区域** 使用效果最佳。
如果配置了 **群攻技能 (AOE)**，在接近目标时也会自动释放群攻技能。

### 使用要求

默认情况下无需更改任何设置。

1. **使用默认主题** -> 选择 “金色” 主题（默认）
2. **启用自动攻击**（默认已启用）

### 最佳优化建议（可选）：

1. **禁用天气、事件特效** 以提高性能
2. **按 <kbd>ESC</kbd> 数次清理 UI 界面**，避免 UI 遮挡
3. **加入 Discord 社区 #How-To 频道** 获取完整的详细设置教程

## 辅助自动化
##### 独立辅助模式：
- 在快捷栏填入**治疗技能**（用于恢复目标生命值）和 **食物/药品**（用于自身恢复）。
- **选择目标角色**（需要跟随的玩家）。
- **按 Z** 自动跟随目标角色。
- **按 ENGAGE** 开始运行。

##### 组队辅助模式：
机器人会自动跟随**队长**，并从队伍窗口自动选择目标进行治疗。

使用步骤：
- **在设置中开启 "组队模式"**
- **将队伍窗口放置在左下角，并尽可能缩小**
- **在启动前关闭所有队伍窗口**
- **按下 ENGAGE** 开始运行。

另外，机器人会**使用 F1 + C (技能栏) 自动施放增益（Buff）技能**，根据设置的“增益间隔时间”触发。

## 快捷栏配置

| 快捷栏图标 | 对应 Flyff 功能 |  说明  |
| ----------- | --------------- | ---------------- |
| 🍔         | 食物             |  快速回复生命值，冷却时间短，当血量低于阈值时触发 |
| 💊         | 药品             |  快速恢复 HP，冷却时间长，触发条件与食物相同 |
| ![](./src/assets/heal_spell_16x16.png) | 治疗技能 | 仅限辅助模式，治疗跟随的目标 |
| 🐶         | 召唤宠物         |  当需要时自动召唤宠物 |
| ![](./src/assets/icon_motion_pickup_16x16.png) | 物品拾取 | 自动捡起掉落物品 |
| ![](./src/assets/icon_refresher_16x16.png) | MP 回复药水 | 快速恢复 MP，低冷却时间，MP 低于阈值时触发 |
| ![](./src/assets/icon_vitaldrink_16x16.png) | FP 回复药水 | 类似 MP 回复，适用于 FP |
| 🗡️         | 攻击技能         | 自动释放攻击技能或**战斗动作** |
| 🪄         | Buff 技能       |  终于等到了这个功能！ |
| ![](./src/assets/rez_spell_16x16.png) | 复活技能 | 仅限辅助模式，复活跟随的目标 |
| ✈️         | 飞行器/坐骑      |  或许以后支持空战？ |

## 自动喊话 (AutoShout)
- 输入你的消息（**每行输入一条消息**，按 Enter 换行）。
- 设置间隔时间。
- 启动后，机器人会按照设定时间间隔自动发送信息。

# 常见问题

**这个软件安全吗？**<br>
是的。如果不信任，可以自行编译源代码，或者 GTFO。

**这是个外挂吗？**<br>
这是一个**带有半自动化功能的 Flyff Universe 客户端**。

<!-- 相关链接 -->
[banner]: ./banner.png
[download]: https://github.com/MadrigalStreetCartel/neuz/releases/
[changelog]: https://github.com/MadrigalStreetCartel/neuz/blob/main/CHANGELOG.md

<!-- 免责声明 -->
<small>免责声明：我们与 Gala Lab Corp.、Sniegu Technologies SAS 或 Flyff Universe **无任何官方关联**。</small>


