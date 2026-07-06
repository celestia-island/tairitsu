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

