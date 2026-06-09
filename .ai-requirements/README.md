# AI 需求文档

本目录用于维护 AI 协作时必须读取和按需同步更新的需求说明。

## 当前结构

- `architecture.md`：整体架构、目录分层、模块边界和协作约束模板。
- `lan-sync.md`：局域网联机协议、状态边界、关键文件和后续扩展约束。
- 其他文档：可按模块、目录、功能域或长期维护主题继续扩展。

## 使用规则

- 具体读取顺序和同步更新规则见 `.github/instructions/ai-requirements-workflow.md`。
- 代码修改后的验证规则见 `.github/instructions/validation-workflow.md`。
- 新增主要模块或长期维护主题时，应同步补充对应需求文档，并更新相关工作流说明。
- 模块职责、脚本映射和模块级 requirements 文档可以在开发推进过程中逐步补齐；一旦形成稳定约定，就应及时写入文档。
