---
description: "适用于 BongoCat 当前 Vue 3 + Vite + Tauri 2 + Rust 项目的开发协作说明。"
---

# BongoCat 项目开发协作说明

## 适用范围

- 适用于当前 BongoCat 桌面应用项目。
- 当前项目不是 Cocos Creator 项目，不要套用 Cocos Creator 2.x/3.x 的组件、资源、生命周期或编辑器约定。
- 本项目主要技术栈为 Vue 3、TypeScript、Vite、Pinia、Vue Router、vue-i18n、UnoCSS、antdv-next、Tauri 2 和 Rust。

## 项目基线

- 前端入口在 `src/main.ts`，根组件为 `src/App.vue`。
- 前端页面主要位于 `src/pages/main/` 和 `src/pages/preference/`。
- 复用组件位于 `src/components/`，组合式逻辑位于 `src/composables/`，Pinia store 位于 `src/stores/`。
- 多语言文案位于 `src/locales/*.json`，语言入口位于 `src/locales/index.ts`。
- Tauri 与 Rust 后端位于 `src-tauri/`，核心逻辑位于 `src-tauri/src/core/`，本地插件位于 `src-tauri/src/plugins/`。
- Tauri 配置分布在 `src-tauri/tauri.conf.json` 和平台配置文件 `src-tauri/tauri.*.conf.json`。

## 前端开发规范

- Vue 文件默认使用 `<script setup lang="ts">` 和 Composition API。
- TypeScript 严格模式开启；不要用 `any` 或非空断言掩盖真实可空路径。
- 路径别名使用 `@/*` 指向 `src/*`。
- 状态管理优先复用现有 Pinia store；不要为单次需求新增全局状态或跨模块临时变量。
- 与 Tauri 能力交互时，优先复用 `src/plugins/` 和 `src/composables/` 中已有封装。
- UI 修改优先沿用 antdv-next、UnoCSS 和现有组件结构；不要引入新的 UI 框架。
- 新增或修改用户可见文案时，同步维护 `src/locales/` 下已有语言 JSON；无法可靠翻译时应明确说明。

## Tauri 与 Rust 规范

- Rust 代码应遵循当前模块划分，优先在已有 `core`、`utils` 或插件模块内扩展。
- 新增前端可调用能力时，应同时检查 Rust command、插件注册、Tauri capability 和前端调用封装。
- 修改窗口、托盘、全局快捷键、输入监听、开机自启、权限请求、自动更新等能力时，必须考虑 macOS、Windows 和 Linux(x11) 差异。
- 平台差异优先使用已有平台文件或 `cfg(target_os = "...")` 处理，不要把单个平台行为写成全局默认。
- 不要新增遥测、远程上报或隐式网络请求；README 声明应用离线运行且不收集用户数据。

## 文件与资源修改原则

- 只修改完成当前任务必须涉及的文件。
- 不要顺手重排无关 import、格式化无关文件或重构相邻代码。
- 不要手动修改构建产物、锁文件或二进制资源，除非任务明确要求且原因充分。
- 修改图标、模型、托盘资源或打包资源时，应同步检查 `scripts/`、`public/`、`src-tauri/assets/` 和 Tauri bundle 配置。

## 常用命令

- 安装依赖：`pnpm install`
- 前端开发：`pnpm dev`
- Tauri 开发：`pnpm tauri dev`
- 前端构建：`pnpm build`
- Tauri 构建：`pnpm tauri build`
- 前端 lint 并自动修复：`pnpm lint`
- Rust 检查：`cargo check --workspace`
- Rust 格式化检查：`cargo fmt --all -- --check`

## 验证要求

- 只改文档时，至少检查内容是否仍符合当前项目结构和技术栈。
- 修改前端 TypeScript、Vue、样式或配置后，优先运行 `pnpm lint`；涉及构建、路由、资源加载或类型边界时运行 `pnpm build`。
- 修改 Rust、Tauri command、插件、权限或配置后，优先运行 `cargo check --workspace`；涉及格式时运行 `cargo fmt --all -- --check`。
- 涉及平台能力的改动，应在最终说明中列出已验证平台和未验证平台。
- 如果验证命令受依赖、网络、权限或系统平台限制无法运行，应明确说明原因和剩余风险。

## 输出要求

- 开始实现前应说明假设、成功标准和验证方式。
- 如果用户请求与当前技术栈或跨平台限制冲突，应直接指出并给出可落地替代方案。
- 最终总结应聚焦实际修改、验证结果和未覆盖风险，不要输出无关背景。
