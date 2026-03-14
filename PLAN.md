# PLAN - 文档体系重构（对齐 hikari 结构）

> 创建时间：2026-03-15
> 基准语言：简体中文（zh-CHS）
> 目标：按隔壁 ../hikari/docs 的多语言目录规范，重构 Tairitsu 文档体系。

## 目标与范围

- 对齐目录范式：docs/<locale>/{guides,system,components}
- 以 zh-CHS 作为完整主文档（source of truth）
- 其他语言先建立同构骨架与占位页，后续逐步翻译
- 保持现有根级文档可访问（改为入口索引）

## 执行阶段

### Phase 1：结构对齐

- [x] 参考 ../hikari/docs 完成目录结构调研
- [x] 创建多语言目录骨架（9 种语言）
- [x] 创建 docs 根入口 docs/README.md

### Phase 2：中文基线文档（完整）

- [x] guides：
  - [x] docs/zh-CHS/guides/index.md
  - [x] docs/zh-CHS/guides/quick-start.md
  - [x] docs/zh-CHS/guides/workspace-map.md
  - [x] docs/zh-CHS/guides/build-test-release.md
  - [x] docs/zh-CHS/guides/migration.md
- [x] system：
  - [x] docs/zh-CHS/system/overview.md
  - [x] docs/zh-CHS/system/runtime.md
  - [x] docs/zh-CHS/system/wit-pipeline.md
  - [x] docs/zh-CHS/system/web-backends.md
  - [x] docs/zh-CHS/system/versioning.md
- [x] components：
  - [x] docs/zh-CHS/components/index.md
  - [x] docs/zh-CHS/components/packages.md

### Phase 3：多语言占位与入口迁移

- [x] 生成非 zh-CHS 语言占位页（guides/system/components）
- [x] 根级迁移入口：docs/migration.md 指向多语言树
- [x] 根级版本入口：docs/versioning.md 指向多语言树

### Phase 4：后续翻译迭代（待执行）

- [ ] en-US 完整翻译（优先）
- [ ] ja-JP / zh-CHT 翻译
- [ ] 其余语种翻译
- [ ] 增加术语对照表与翻译一致性检查

## 当前状态

- 总体状态：已完成第一阶段重构（结构 + 简中主文档 + 多语言骨架）
- 下一步：进入 Phase 4，按语言批次补全文档。

## 变更日志（实时）

- 2026-03-15 结构对齐完成：建立 docs/<locale>/{guides,system,components}
- 2026-03-15 简中主文档完成：12 个主题文档 + 中文入口
- 2026-03-15 根级迁移：docs/migration.md、docs/versioning.md 改为入口页
- 2026-03-15 多语言占位完成：en-US/ja-JP/zh-CHT/ko-KR/fr-FR/es-ES/ru-RU/ar-SA
- 2026-03-15 README 文档入口更新：新增 docs 根入口与 zh-CHS 主入口链接
