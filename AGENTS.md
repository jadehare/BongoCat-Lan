# 仓库说明

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## 5. Read Repository Instructions First

在本仓库内进行任何分析、修改或新增代码前，必须先读取并遵守以下说明文件：

- `.github/copilot-instructions.md`
- `.github/instructions/` 目录下当前存在、且与任务相关的 Markdown 说明文件

规则：

- `.github/instructions/` 是动态说明目录，不要假设当前文件列表是固定的；执行任务时默认读取相关 `.md` 文件。
- 如果无法判断某个 `.github/instructions/*.md` 是否相关，应读取该文件；如果仍无法判断，应读取全部 `.github/instructions/*.md`。
- 后续如果在 `.github/instructions/` 下新增说明文件，应自动纳入本仓库规范，不需要再修改本文件。
- 如果多个说明文件同时适用，优先遵循与当前任务更具体、更直接相关的说明。
- 如果不同说明之间存在冲突，且无法明确判断优先级，应先向用户确认，再继续执行。
