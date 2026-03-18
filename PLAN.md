# Tairitsu 改进计划

## 1. Props DSL 宏增强 ✅ 已完成

详见 git log。

## 2. CSS 基础设施包 (tairitsu-style)

### 背景

当前 hikari 中存在大量 CSS 相关的基础设施代码，应当作为 tairitsu 框架的一部分提供给所有前端依赖使用：

1. **StyleStringBuilder** - 构建 CSS style 字符串的工具
2. **ClassesBuilder** - 构建 CSS class 字符串的工具
3. **CssProperty 枚举** - 类型安全的 CSS 属性名

### 目标

创建 `packages/style` 子包，提供：

```
tairitsu-style/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── properties/
│   │   ├── mod.rs
│   │   ├── css.rs          # CssProperty 枚举 (全部 W3C 标准属性)
│   │   └── generated.rs    # 自动生成的属性定义
│   ├── builders/
│   │   ├── mod.rs
│   │   ├── style.rs        # StyleStringBuilder
│   │   └── classes.rs      # ClassesBuilder
│   ├── traits/
│   │   ├── mod.rs
│   │   └── utility.rs      # UtilityClass trait
│   └── css_data/           # W3C CSS 规范数据
│       ├── index.json      # 属性索引
│       └── ...
└── build.rs                # 从 W3C 规范生成属性的脚本
```

### 实现步骤

#### Phase 1: 基础结构 (手动)

- [ ] 创建 `packages/style` 包
- [ ] 迁移 `StyleStringBuilder` 从 hikari-animation
- [ ] 迁移 `ClassesBuilder` 和 `UtilityClass` trait 从 hikari-palette
- [ ] 迁移现有 `CssProperty` 枚举

#### Phase 2: W3C CSS 属性自动生成 (爬虫)

- [ ] 研究 W3C CSS 规范数据源
  - MDN CSS Reference: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference
  - W3C CSS Index: https://w3c.github.io/csswg-drafts/indexes.html
- [ ] 创建爬虫脚本获取所有标准 CSS 属性
- [ ] 生成 `CssProperty` 枚举，包含：
  - 属性名 (kebab-case)
  - 属性分类 (Layout, Box Model, Typography, etc.)
  - 是否为简写属性
  - 初始值（可选）
- [ ] build.rs 集成，自动生成 `generated.rs`

#### Phase 3: 增强功能

- [ ] 添加 CSS 值类型枚举（常用值如 `auto`, `none`, `inherit`）
- [ ] 添加属性验证（可选）
- [ ] 支持 CSS 变量类型提示

### 数据源选项

1. **MDN Browser Compatibility Data (BCD)**
   - 仓库：https://github.com/mdn/browser-compat-data
   - 包含所有 CSS 属性及其兼容性数据
   - 可通过 `@mdn/browser-compat-data` npm 包获取

2. **W3C CSSWG Drafts**
   - https://w3c.github.io/csswg-drafts/indexes.html
   - 官方规范索引

3. **CSSTree**
   - https://github.com/csstree/csstree
   - 包含完整的 CSS 语法数据

### 示例：生成的代码

```rust
// generated.rs - 自动生成，不要手动编辑
// Source: MDN CSS Reference + W3C CSSWG Indexes

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssProperty {
    // Layout
    Display,
    Position,
    Top,
    Right,
    Bottom,
    Left,
    ZIndex,
    Float,
    Clear,
    // ... 数百个属性

    // CSS Values and Units Module Level 4
    AccentColor,
    // ...

    // 新增实验性属性（标记）
    #[cfg(feature = "experimental")]
    ViewTimeline,
}

impl CssProperty {
    /// Get the CSS property name in kebab-case
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::Position => "position",
            // ... 自动生成
        }
    }

    /// Get the category of this property
    pub const fn category(&self) -> CssCategory {
        match self {
            Self::Display => CssCategory::Layout,
            Self::Margin => CssCategory::BoxModel,
            Self::FontSize => CssCategory::Typography,
            // ...
        }
    }
}
```

### API 示例

```rust
use tairitsu_style::{StyleStringBuilder, CssProperty};

let style = StyleStringBuilder::new()
    .add(CssProperty::Display, "flex")
    .add_px(CssProperty::Gap, 8)
    .add_custom("--custom-var", "value")
    .build_clean();
// Output: "display:flex;gap:8px;--custom-var:value"
```

```rust
use tairitsu_style::{ClassesBuilder, UtilityClass};

let classes = ClassesBuilder::new()
    .add(MyClass::Button)
    .add_if(MyClass::Active, || is_active)
    .build();
```

## 验收标准

- [ ] `tairitsu-style` 包编译通过
- [ ] 包含全部 W3C 标准 CSS 属性（~300+）
- [ ] `StyleStringBuilder` 功能完整
- [ ] `ClassesBuilder` 功能完整
- [ ] hikari 成功迁移使用
