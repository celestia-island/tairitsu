# tairitsu — 项目状态与计划 (PLAN)

> 本文件记录项目当前状态、近期进展与后续计划。最近一次人工刷新：**2026-07-04**。

## 1. 项目概述

- **名称**：`tairitsu`
- **简介**：基于 WASM Component Model 的全栈 Web 框架（monorepo）。
- **远程仓库**：https://github.com/celestia-island/tairitsu
- **技术栈**：Rust / Node/TypeScript / just
- **类别**：platform

## 2. 当前状态

- **当前分支**：`dev`
- **工作区**：有未提交改动（本次发布元数据完善，见第 3 节）
- **最近提交**：`760aa9a2` docs: add lagrange docs deployment CI（2026-07-04）
- **分支对比**：`dev` 领先 `master` 1621 个提交

## 3. 未提交改动明细

本次改动为完善 crates.io 发布元数据，文件清单：

```
 M README.md                  # 顶部 badge 区新增 docs.rs 官方徽章
 M packages/runtime/Cargo.toml  # 主包：补 keywords / categories / [package.metadata.docs.rs]
 M PLAN.md                    # 本次状态刷新
```

> 注：此前 PLAN 记录的「31 项 browser-glue 改动」已于提交 `4fea511d`（feat(browser-glue): regenerate browser glue bindings）全部提交，工作区随之干净。本节为新一轮改动。

## 4. 近期进展（最近提交）

- `760aa9a2` docs: add lagrange docs deployment CI
- `4fea511d` feat(browser-glue): regenerate browser glue bindings
- `30144584` docs: add PLAN.md current-status snapshot
- `03e80126` fix(mcp): system-fonts-first + install rustls crypto provider before reqwest
- `16ab5dee` chore: accumulative health-check fixes across workspace
- `1b295f8b` fix(mcp): scale fonts to supersampled resolution + bump output to desktop size
- `1a456541` fix(mcp): re-enable kou font-fetch — async load path is now runtime-safe
- `86854d78` style(mcp): wrap render_png call to satisfy rustfmt

## 5. 发布元数据完善进度

- [x] README.md 顶部新增 docs.rs 官方徽章（`[![docs.rs](https://docs.rs/tairitsu/badge.svg)](https://docs.rs/tairitsu)`，采用与现有徽章一致的 HTML 写法）
- [x] 主包 `tairitsu` 补 `keywords`（wasm / wit / component-model / runtime / webassembly）
- [x] 主包 `tairitsu` 补 `categories`（wasm / api-bindings）
- [x] 主包 `tairitsu` 补 `[package.metadata.docs.rs]`（`all-features = true` + 主流目标 triples）

## 6. 验证结果

- `cargo +nightly check --workspace`：通过（`tairitsu` 主包编译正常）
- `cargo +nightly clippy --workspace -- -D warnings`：通过
- `cargo +nightly test --lib`：通过；仅 `tairitsu-browser-wit-resolver` 存在与本次改动无关的并行 env 变量竞态（`--test-threads=1` 下全部通过）

> 环境说明：本机 stable rustc 为 1.90，低于 wasmtime 43 / cranelift 所需的 1.91，故验证使用 nightly（1.93）。此外 `examples/website` 依赖的上游 `hikari`（dev 分支）当前 `tairitsu-hooks` 解析失败，属预先存在的上游问题，与本次元数据改动无关；验证时已临时排除该 example 并随后还原。

## 7. 后续计划

1. 推进核心功能里程碑，收敛历次审计（R3/R4/安全审计）遗留项。
2. 保持 workspace 内各 crate 一致（Cargo.lock、rust-toolchain、deny.toml）。
3. 跟进上游 `hikari` dev 分支的 `tairitsu-hooks` 解析问题，恢复 `examples/website` 全量构建。
4. 评估为本仓库固定 `rust-toolchain`（≥1.91）以避免 MSRV 漂移。
5. 定期刷新本 PLAN.md 以反映最新状态。


---

## 维护记录（2026-07-10）

### 待办：缺少 PRIVACY.md / TERMS.md 法律文档

`docs/ja/security.md`（第 114-115 行）的页脚链接了 `PRIVACY.md`（隐私政策）和 `TERMS.md`（利用规约），但这两个文件在仓库中任何位置都不存在。英文版 `docs/en/security.md` 没有这些链接（ja 是唯一引用它们的语言）。

这是一个内容缺口：需要撰写实际的隐私政策和服务条款文本，并决定是否需要多语言版本（目前只有 ja 引用）。在此之前，这两个链接会 404。

#### 状态（已处理）

已删除 `docs/ja/security.md` 中指向不存在文件的 `PRIVACY.md` 和 `TERMS.md` 死链（英文版 security.md 本就没有这些链接，说明它们是 ja 独有的悬空引用）。如果未来确实需要隐私政策/服务条款，请先撰写 `PRIVACY.md` / `TERMS.md`（放仓库根目录），再让各语言的 security.md 引用。

### 本次维护已完成

- 修正各语言 README 的 SySL 许可证链接（`./LICENSE.txt` → `./LICENSE`）。
- 修正 5 个 package README 中 `docs/en-US/` → `docs/en/`。
- 将未翻译指南的链接（getting-started、vdom、dioxus 迁移、debug-agent、企业支持）改为指向英文原文。
- 修正 zh-Hans quick-start 中 examples 链接指向实际存在的 examples 目录。

## 维护记录（2026-07-10，第二轮）

### 待办：generator/config.py 有 30 个重复键冲突（真实 bug）

`scripts/generator/config.py`（7788 行的浏览器绑定生成配置）中 ruff 报告 838 个 F601（multi-value-repeated-key-literal）。经分析：

- **808 个是同值重复**（同一个键在多个章节中出现，值相同）—— 无害但冗余，可安全删除后者。
- **30 个是异值冲突**（同一键被定义两次，值不同）—— 后者会静默覆盖前者，第一个映射丢失。这是真实 bug，例如：

| 键 | 第一次值 | 第二次值（覆盖） |
|----|---------|----------------|
| `('service-worker-container','getController')` | `'any'` | `'service-worker'` |
| `('service-worker-container','getReady')` | `'promise-any'` | `'service-worker-registration'` |
| `('rtc-sctp-transport','getTransport')` | `'rtc-ice-transport'` | `'rtc-dtls-transport'` |
| `('css-style-declaration','set-property','priority')` | `True` | `'string'` |
| `('window','scroll','options')` | `'dictionary:ScrollToOptions | undefined'` | `'any'` |
| `('document','write','text')` | `'string-from-array'` | `'string'` |
| ...（共 30 个） | | |

#### 为什么没有自动修复

每个冲突需要熟悉浏览器绑定生成意图的维护者判断哪个值是正确的（第一次还是第二次）。盲目删除任一方都可能改变生成输出。建议：

1. 由维护者逐个确认 30 个异值冲突的正确值。
2. 删除 808 个同值重复（安全）。
3. 修复后 ruff F601 计数将归零。

可用 `python -c "import ast; ..."` 脚本（见本轮维护）重新检测冲突。

### 维护记录补充（2026-07-10）

尝试用脚本删除重复键行，但简单的行删除会破坏多行条目和尾随逗号
（导致 `IndentationError`），已回退。这些重复键需要逐条目语义级编辑
（而非按行删除），并配合生成器输出对比验证。维持现状（已在上方记录）。
