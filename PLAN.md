# Tairitsu Website Demo 开发计划

> 照抄 Hikari 的 website demo 设计，通过 Cargo.toml patch 引用 hikari 库

## 进度

- [x] 探索 hikari 和 tairitsu 项目结构
- [x] 配置 Cargo.toml patch 引用 hikari
- [x] 复制/创建样式系统
- [x] 重构 website 组件结构
- [x] 创建文档页面
- [x] 整合现有 docs/ 内容

## 已完成任务

### 1. Cargo.toml 配置
- [x] 添加 hikari-palette 依赖（通过直接路径引用）
- [x] 配置 package.metadata.tairitsu 路由（hash routing）

### 2. 样式系统
- [x] 创建 CSS 样式文件 `public/styles.css`
- [x] Tairitsu（对立）深色主题配色
- [x] 顶部导航栏样式 `.tairitsu-topnav`
- [x] 侧边栏样式 `.tairitsu-sidebar`
- [x] 页面布局样式 `.tairitsu-page`
- [x] 卡片/按钮等组件样式

### 3. 组件结构
- [x] `components/mod.rs` - top_nav(), sidebar(), code_block()
- [x] 照抄 Hikari 的导航组件设计

### 4. 页面结构
- [x] `pages/home.rs` - 首页（Hero + 特性介绍）
- [x] `pages/guides/` - 指南文档页
  - [x] mod.rs - 概览页
  - [x] quick_start.rs - 快速开始
  - [x] workspace_map.rs - 工作区地图
  - [x] build_test_release.rs - 构建/测试/发布
  - [x] migration.rs - 迁移指南
  - [x] glossary.rs - 术语对照表
- [x] `pages/system/` - 系统文档页
  - [x] mod.rs - 概览页
  - [x] overview.rs - 系统总览
  - [x] runtime.rs - 运行时
  - [x] wit_pipeline.rs - WIT 流水线
  - [x] web_backends.rs - Web 后端
  - [x] versioning.rs - 版本策略
- [x] `pages/packages/` - 包文档页
  - [x] mod.rs - 包总览和清单
- [x] `pages/not_found.rs` - 404 页面

### 5. 路由配置
- [x] 配置 hash routing（在 Cargo.toml 中）
- [x] JavaScript 页面切换逻辑

## 文件结构

```
examples/website/
├── Cargo.toml          # 配置依赖和路由
├── public/
│   └── styles.css      # 主样式文件
└── src/
    ├── lib.rs          # 入口
    ├── app.rs          # 应用根组件
    ├── components/
    │   └── mod.rs      # top_nav, sidebar, code_block
    └── pages/
        ├── mod.rs
        ├── home.rs
        ├── not_found.rs
        ├── guides/
        │   ├── mod.rs
        │   ├── quick_start.rs
        │   ├── workspace_map.rs
        │   ├── build_test_release.rs
        │   ├── migration.rs
        │   └── glossary.rs
        ├── system/
        │   ├── mod.rs
        │   ├── overview.rs
        │   ├── runtime.rs
        │   ├── wit_pipeline.rs
        │   ├── web_backends.rs
        │   └── versioning.rs
        └── packages/
            └── mod.rs
```

## 设计参考

### Hikari Website 架构
```
app.rs
  └── top_nav()
  └── sidebar()
      └── main.content
          └── home::render()
          └── guides::render_all()
          └── system::render_all()
          └── packages::render_all()
          └── not_found::render()
```

### Tairitsu 文档导航结构
```
Home
Guides
  - 快速开始
  - 工作区地图
  - 构建/测试/发布
  - 迁移指南
  - 术语对照表
System
  - 系统总览
  - 运行时
  - WIT 流水线
  - Web 后端
  - 版本策略
Packages
  - 包总览
  - 包清单
```

## 下一步

1. **构建验证** - 运行 `just dev` 验证编译
2. **功能测试** - 在浏览器中测试路由和页面切换
3. **内容完善** - 根据实际 docs/ 内容更新页面
4. **多语言支持** - 添加 i18n 系统
