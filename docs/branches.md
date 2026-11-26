# 分支清单

开发、同步及发版流程见 [AGENTS.md](../AGENTS.md#分支与发布)。

状态截至 2026-11-26。custom 已同步上游 main 的 `b47651700`。"自有提交"指不属于上游 main 的提交数。

### 已合入 release/custom

| 分支 | 用途 | 上游 PR | 备注 |
|---|---|---|---|
| feat/btw-side-question | 会话旁路提问 | [#1251](https://github.com/spacering-net/codeg/pull/1251) open | 0.48.5 同步上游侧栏名称和迁移 |
| feat/enter-steer-f2-rename | 回车转向排队消息、F2 重命名 | [#1208](https://github.com/spacering-net/codeg/pull/1208) open | 已合入 |
| feat/live-output-auto-reload | 输出文件实时自动重载 | [#775](https://github.com/spacering-net/codeg/pull/775) open | 0.48.5 适配上游活动栏 |
| fix/sidebar-subsession-disclosure | 子会话折叠箭头配色修复 | [#1122](https://github.com/spacering-net/codeg/pull/1122) open | 已合入 custom；上游 PR 仍开着 |
| feat/aux-collapsed-tab-rail | 辅助面板折叠 tab 栏 | [#1258](https://github.com/spacering-net/codeg/pull/1258) open | 已合入共享侧栏按钮更新 |
| feat/project-knowledge | 共享项目笔记与计划 | [#1299](https://github.com/spacering-net/codeg/pull/1299) open | 0.48.5 合入笔记 Markdown、计划与看板任务关联及 MCP 指引 |
| feat/session-messaging | 独立会话创建、通信、自动关联、运行时选项 | [#1207](https://github.com/spacering-net/codeg/pull/1207) open | 已合入，默认开启通信，并更新协作与 steer 说明 |
| feat/session-history | 分页和关键词查询历史 | [#1313](https://github.com/spacering-net/codeg/pull/1313) 草稿 | 已合入分页查询和会话恢复说明 |
| feat/delegation-delivery-core | 委派结果持久回投和持续上报 | [#1302](https://github.com/spacering-net/codeg/pull/1302) 草稿 | 已合入，依赖 #1207 |
| feat/session-progress-stacked | 会话 Brief、Digest 和手动协作关系管理 | [#1303](https://github.com/spacering-net/codeg/pull/1303) 草稿 | 0.48.5 合入可折叠进度、Brief Markdown 和长程任务指引，依赖 #1302 |
| feat/delegation-model-options | 委派时指定模型和推理档位 | [#1304](https://github.com/spacering-net/codeg/pull/1304) 草稿 | 已合入，依赖 #1302，复用 list_agent_options 与 ACP 启动配置 |
| fix/personal-agent-aux-path | Windows 兼容的个人助理侧栏目录 | 无 | 已合入，目录统一为上游的 `aux-parts` |

### 未合入 release/custom

| 分支 | 用途 | 上游 PR | 自有提交 | 备注 |
|---|---|---|---|---|
| feat/acp-close-session-on-disconnect | ACP 断连时关闭 agent 侧会话 | [#1030](https://github.com/spacering-net/codeg/pull/1030) open | 1 | |
| feat/acp-delete-session-on-disconnect | 删除会话时丢弃 agent 侧会话 | [#1031](https://github.com/spacering-net/codeg/pull/1031) open | 2 | 建立在 close 分支之上，先合 close |
| feat/codeg-static-dir | CODEG_STATIC_DIR 加载 web 静态目录 | [#829](https://github.com/spacering-net/codeg/pull/829) open | 2 | |
| feat/rename-session | MCP 重命名当前/子会话 | 无 | 1 | 分支基线含上游 0.48.1，custom 追平后差异缩小到自有提交；内容是 session-messaging 的子集，可只留 PR 分支 |
| fix/steer-turn-settings | Codex 转向/设置变更后的回合重启 | [#1298](https://github.com/spacering-net/codeg/pull/1298) closed 未合并 | 2 | 同上；贡献是否放弃待用户定，放弃则转私有分支按 custom 同步 |
| feat/aux-subagent-panel-two-column | 子智能体面板双列布局 | 无 | 3 | 2026-09-20 后无更新，先评估去留 |
| pr/render-page | MCP 渲染可应答页面卡片 | [#1061](https://github.com/spacering-net/codeg/pull/1061) open | 3 | 贡献分支 |
| integration/long-task | 长程协作联调 | 无 | — | 联调分支，功能已通过正式分支合入 |


已有合入记录的分支继续同步，新增功能分支按最近 24 小时的创建时间纳入。已关闭且未合并的 PR 及其定制版本排除。已由正式分支覆盖的试验或旧实现保留作记录。

## 维护规则

- PR 一律开到上游 `spacering-net/codeg`，源分支推到 `leehom0123/codeg`。默认按上游仓库查找 PR 编号。
- 新建开发分支：在"未合入"表中加一行，写清用途。
- 分支合入或被放弃：移到"已合入"表（或直接删除，注明版本号），更新备注。
- 在上游开 PR 或 PR 状态变化（合并、关闭）：更新"上游 PR"列的链接和状态。PR 被上游合并后，custom 下次合 main 即自动对齐，不需要额外动作。
- 每次走完上面的上游同步流程：更新"状态截至"日期，并按需修正"自有提交"数。

## 长程协作合并顺序

先合 #1207，再合 #1302。#1303 和 #1304 随后可并行合入。#1299 和 #1313 可独立合入。每个交付分支对应一个上游 PR，验收清单见 [#1300](https://github.com/spacering-net/codeg/issues/1300)。
