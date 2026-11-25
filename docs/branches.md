# 分支模型与上游同步

## 分支模型

- `origin` 指上游仓库，原名 xintaofei/codeg，已迁移为 **spacering-net/codeg**（远端 URL 自动重定向）。`fork` 指 leehom0123/codeg（本地分支的推送目标）。
- `main`：跟踪 `origin/main`，不做本地开发。
- `release/custom`：定制集成分支，也是 fork 的默认分支。定制侧与上游的冲突在这里解决一次。
- 开发分支分两种角色，同步基准不同：
  - **贡献分支**：在上游开着 PR（head 就是推到 fork 的同名分支，base=main）。同步基准是 `origin/main`——把 main 合进分支不会污染 PR diff，反而让 PR 无冲突；合 custom 会把私有提交带进 PR。
  - **私有分支**：不走上游 PR，只合 `release/custom`。同步基准是 custom，不碰上游。

```
origin/main ──> main ──merge──> release/custom <──merge── 私有分支
                    └──────────merge（同步）────────────── 贡献分支（带 open PR）
```

## 上游 main 更新时的处理流程

```bash
git fetch origin
git checkout main
git merge --ff-only origin/main          # 本地 main 追平上游
git checkout release/custom
git merge main                           # 与上游的冲突只在这里解决一次
git push fork release/custom
```

对尚未合回的开发分支：

- 近期就要合回的：不预先同步，直接在 custom 上 `git merge <分支>`，冲突现场解决。
- 还要继续开发的私有分支：在分支上 `git merge release/custom`，以 custom 为基准同步。此后该分支不会再撞上游冲突。
- 贡献分支（有 open PR）：在分支上 `git merge origin/main`。PR 的 diff 只算 head 相对 base 的净变化，合 main 只会消掉 PR 页面上的冲突，不会引入无关提交；这类分支不要合 custom。

两条纪律：

1. 私有分支不自行合并 `origin/main`；贡献分支不合并 `release/custom`。基准混用要么让上游 PR 夹带私有提交，要么让同一批上游冲突在多个分支各解一遍。
2. 建议开启 `git config rerere.enabled true`，重复出现的冲突（i18n 语言包、sidebar 组件）会自动复用上次的解决方案。custom 侧和贡献分支侧的同类冲突解法能互相复用。

## 分支清单

状态截至 2026-11-25。custom 已同步上游 main 的 `68b17041e`。"自有提交"指不属于上游 main 的提交数。

### 已合入 release/custom

| 分支 | 用途 | 上游 PR | 备注 |
|---|---|---|---|
| feat/btw-side-question | 会话旁路提问 | [#1251](https://github.com/spacering-net/codeg/pull/1251) open | 已合入 |
| feat/enter-steer-f2-rename | 回车转向排队消息、F2 重命名 | [#1208](https://github.com/spacering-net/codeg/pull/1208) open | 已合入 |
| feat/live-output-auto-reload | 输出文件实时自动重载 | [#775](https://github.com/spacering-net/codeg/pull/775) open | 已合入 |
| fix/sidebar-subsession-disclosure | 子会话折叠箭头配色修复 | [#1122](https://github.com/spacering-net/codeg/pull/1122) open | 已合入 custom；上游 PR 仍开着 |
| feat/aux-collapsed-tab-rail | 辅助面板折叠 tab 栏 | [#1258](https://github.com/spacering-net/codeg/pull/1258) open | 已合入共享侧栏按钮更新 |
| feat/project-knowledge | 共享项目笔记与计划 | [#1299](https://github.com/spacering-net/codeg/pull/1299) open | 已合入，知识页签共用侧栏按钮 |
| feat/session-messaging | 会话间消息：转向、链接、读历史、运行时选项 | [#1207](https://github.com/spacering-net/codeg/pull/1207) open | 已合入最新会话通信与上游冲突修复 |
| feat/delegation-delivery-core | 委派结果持久回投和持续上报 | [#1302](https://github.com/spacering-net/codeg/pull/1302) 草稿 | 已随会话进度合入 |
| feat/session-progress-stacked | 会话 Brief、Digest 和协作关系侧栏 | [#1303](https://github.com/spacering-net/codeg/pull/1303) 草稿 | 已合入会话进度 |
| feat/delegation-model-options | 委派时指定模型和推理档位 | [#1304](https://github.com/spacering-net/codeg/pull/1304) 草稿 | 已合入，复用 list_agent_options 与 ACP 启动配置 |
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
