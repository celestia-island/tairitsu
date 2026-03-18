# Tairitsu 改进计划

## 1. Props DSL 宏增强

### 问题

当前 Props 定义冗长繁琐：

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AvatarProps {
    #[props(default)]
    src: Option<String>,

    #[props(default = "Avatar".to_string())]
    alt: String,

    #[props(default = AvatarSize::Md)]
    size: AvatarSize,

    #[props(default = AvatarVariant::Circular)]
    variant: AvatarVariant,

    #[props(default)]
    fallback: Option<String>,

    #[props(default)]
    fallback_mode: AvatarFallbackMode,

    #[props(default)]
    class: String,
}
```

### 解决方案

创建 `#[define_props]` 宏，支持简洁的 DSL 语法：

```rust
#[define_props]
pub struct AvatarProps {
    src: Option<String> = None,
    alt: String = "Avatar".to_string(),
    size: AvatarSize = AvatarSize::Md,
    variant: AvatarVariant = AvatarVariant::Circular,
    fallback: Option<String> = None,
    fallback_mode: AvatarFallbackMode = AvatarFallbackMode::default(),
    class: String = String::new(),
}
```

### 宏展开结果

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AvatarProps {
    #[props(default)]
    pub src: Option<String>,

    #[props(default = "Avatar".to_string())]
    pub alt: String,

    #[props(default = AvatarSize::Md)]
    pub size: AvatarSize,

    // ...
}

impl Default for AvatarProps {
    fn default() -> Self {
        Self {
            src: None,
            alt: "Avatar".to_string(),
            size: AvatarSize::Md,
            // ...
        }
    }
}
```

### 实现步骤

- [ ] 修改 `packages/macros/src/lib.rs` 添加 `define_props` 宏
- [ ] 创建 `packages/macros/src/props_dsl.rs` 实现解析逻辑
- [ ] 支持 `Option<T>` 自动推断 `#[props(default)]`
- [ ] 支持字面量默认值
- [ ] 支持表达式默认值
- [ ] 生成 `Default` impl
- [ ] 在 hikari-components 中验证

### 语法糖扩展

可选的简化语法：

```rust
// 使用 ? 后缀表示 Option
#[define_props]
pub struct AvatarProps {
    src?: String,              // 等价于 src: Option<String> = None
    alt: String = "Avatar",    // 字面量
    size: AvatarSize = Md,     // 枚举变体简写
    class: String,             // 无默认值 = 空字符串
}
```

## 验收标准

- [ ] 宏编译通过
- [ ] 生成的 Props 结构体与手动编写的功能等价
- [ ] hikari-components 中至少 5 个组件使用新宏
- [ ] 代码行数减少 > 30%
