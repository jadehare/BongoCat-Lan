# 仓库级 AI 协作说明

本仓库用于维护 AI 协作规范、instruction、prompt 和需求文档模板。

## 默认协作原则

- 在开始分析、修改或新增内容前，先读取本文件以及 `.github/instructions/` 下相关说明。
- 如果任务涉及特定技术栈或运行环境还应补充读取对应技术栈说明文件；当前仓库默认重点覆盖 Cocos Creator 2.4.15。
- 优先做小范围、可验证、低风险的改动。
- 不要把某个具体项目的一次性实现细节沉淀成仓库级长期规范。
- 如果规则只适用于某个项目，应在对应项目内单独维护，不要直接写进通用模板仓库。

## AI 需求文档规则

- AI 协作需求统一维护在 `ai-requirements/`。
- 进行任何代码分析、修改、新增脚本或重构前，必须重新读取 `.github/copilot-instructions.md` 与 `.github/instructions/` 下当前相关说明，不要跳过这一步，也不要凭上一次任务记忆直接开始写代码。
- 进行代码分析、修改、新增脚本或重构前，优先读取 `ai-requirements/` 下与当前任务相关的文档。
- 具体读取顺序和同步更新规则见 `.github/instructions/ai-requirements-workflow.md`。
- TypeScript 修改后的验证规则见 `.github/instructions/validation-workflow.md`。

## 交互约定

### Checklist

- checklist 默认放在项目根目录的 `.checklists` 文件夹下。
- 整体 checklist 用于跨模块长期跟踪，优先使用 `.checklists/GAME_CHECKLIST.md`；保留完整历史，完成项统一用 `[x]` 标记，不删除已完成条目。
- 模块或当前任务 checklist 用于短期执行，优先使用 `.checklists/<module-or-task>-checklist.md`；开发过程中同样保留完成项并用 `[x]` 标记，待该模块全部完成且整体进度已同步后再删除对应文件。
- 同一事项若同时出现在整体 checklist 和模块 checklist 中，完成后两个文件都要同步勾选；模块 checklist 在全部完成后删除。
- 用户说“更新 checklist”时，先判断是在更新整体进度还是当前模块进度；如果无法判断，默认同时检查整体和模块 checklist，并在说明中明确假设。
- 用户说“完成任务”时，默认优先按最近一次相关模块 checklist 执行；如果没有模块 checklist，再回退参考整体 checklist。

### Commit

- 如果用户输入 `commit`，默认表示：仅基于当前 staged diff 生成 commit message 并提交。
- 执行 `commit` 时，不自动暂存未暂存或未跟踪文件。
- 如果 staged 为空，应明确提示无法提交。
- commit message 保持简短直接，优先使用 `feat:`、`fix:`、`refactor:`、`docs:` 等格式，并使用中文；标题应聚焦本次提交的单一目的，不要堆砌实现细节、验证过程或文件清单。
- 当输入 `commit -all`、`提交 -all` 或 `cm -all` 时，默认行为是：查看当前所有 staged、unstaged 和 untracked 改动，按内容相关性决定一次或多次提交，并在提交前主动暂存对应文件。
- 执行 `commit -all` 时，如果改动明显属于多个独立主题，应拆分多次提交；如果属于同一任务或强依赖链路，可合并为一次提交。
- 如果改动交叉在同一文件且无法安全拆分，或拆分边界不明确，应先询问用户，不要主观拆分。
- 如果存在部分暂存场景，应先判断是否会破坏用户原有 staged 边界；除非用户明确要求，否则不要无意改写其暂存选择。
- 生成 commit message 或总结变更时，默认应查看并总结所有可读的文本类文件、配置文件、JSON 文件和开发脚本文件的具体内容，例如 `.ts`、`.js`、`.mjs`、`.cjs`、`.json`、`.md`、`.yml`、`.yaml`、`.toml`、`.ini`、`.env*`、`.sh`、`.bat`、`.ps1` 等。
- 生成 commit message 或总结变更时，不要查看或总结 Cocos Creator 资源文件、二进制资源和美术资源的具体内容，例如 `.prefab`、`.scene`、`.fire`、`.png`、`.jpg`、`.jpeg`、`.webp`、`.gif`、`.psd`、`.atlas`、`.skel`、Spine 导出资源等；这类文件默认只根据文件名和路径判断用途，除非用户明确要求查看。

## 文档维护原则

- 优先维护稳定、可迁移、可复用的规则。
- 项目强相关的架构说明、资源命名、目录映射、节点依赖或业务链路，不应直接作为通用模板保留在仓库里。
- 如果需要保留示例，应将其写成“示例”或“模板”，而不是写成默认事实。

## 安全词握手

- 每次响应第一行必须输出：`[SAFEWORD: SMITH-OK]`
- 若未输出该安全词，视为未通过指令握手，本次结果无效
- 安全词只输出一次，且必须在正文前
