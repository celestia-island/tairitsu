# PLAN

## 文档体系重构（对齐 hikari 结构）

### 目标

- [x] 对齐目录范式：docs/<locale>/{guides,system,components}
- [x] 以 zh-CHS 作为完整主文档基线
- [x] 建立多语言文档入口并完成多语种扩展
- [x] 保持根级文档入口可访问

### 已完成范围

- [x] 结构对齐：多语言目录与 docs 根入口
- [x] zh-CHS 完整文档：guides/system/components 全套
- [x] en-US 完整文档：guides/system/components 全套
- [x] ja-JP 与 zh-CHT 文档：guides/system/components 全套
- [x] 其他语种入口本地化：fr-FR/es-ES/ko-KR/ru-RU/ar-SA
- [x] 术语一致性：新增 zh-CHS 术语对照表并接入导航

### 验证与质量门

- [x] cargo check --workspace
- [x] cargo clippy --workspace --all-targets --all-features -- -D warnings
- [x] cargo test --workspace
- [x] npm run typecheck (packages/browser-glue)
- [x] cargo test -p tairitsu-e2e

### 当前结论

- [x] PLAN 中所有任务已完成
