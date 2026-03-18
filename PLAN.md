# Tairitsu 改进计划

## 1. Props DSL 宏增强 ✅ 已完成

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

impl Default for AvatarProps {
    fn default() -> Self {
        Self {
            src: None,
            alt: "Avatar".to_string(),
            size: AvatarSize::Md,
            variant: AvatarVariant::Circular,
            fallback: None,
            fallback_mode: AvatarFallbackMode::default(),
            class: String::new(),
        }
    }
}
```

### 解决方案 ✅ 已实现

创建 `#[define_props]` 宏，使用 `#[default(...)]` 属性语法：

```rust
#[define_props]
pub struct AvatarProps {
    src: Option<String>,                    // Option 自动默认为 None
    #[default("Avatar".to_string())]
    alt: String,
    #[default(AvatarSize::Md)]
    size: AvatarSize,
    #[default(AvatarVariant::Circular)]
    variant: AvatarVariant,
    fallback: Option<String>,               // 自动推断
    fallback_mode: AvatarFallbackMode,      // 使用 Default trait
    class: String,                          // String 自动默认为 String::new()
}
```

### 实现细节

由于 Rust 的 `field: Type = value` 语法目前是实验性功能（E0658），我们采用 `#[default(value)]` 属性语法：

- `#[default(expr)]` - 显式指定默认值
- 无属性 - 根据类型自动推断：
  - `Option<T>` → `None`
  - `String` → `String::new()`
  - `Vec<T>` → `Vec::new()`
  - 其他 → `Default::default()`

### 实现步骤 ✅

- [x] 修改 `packages/macros/src/lib.rs` 添加 `define_props` 宏
- [x] 创建 `packages/macros/src/props_dsl.rs` 实现解析逻辑
- [x] 支持 `#[default(...)]` 属性指定默认值
- [x] 支持 `Option<T>` 自动推断 `#[props(default)]` + `None`
- [x] 支持 `String` 自动推断 `#[props(default)]` + `String::new()`
- [x] 支持 `Vec<T>` 自动推断 `#[props(default)]` + `Vec::new()`
- [x] 支持其他类型使用 `Default::default()`
- [x] 生成 `Default` impl
- [x] 在 hikari-icons 中验证（IconProps）

### 示例：IconProps

**之前（27行）：**
```rust
#[cfg(feature = "tairitsu")]
#[derive(Clone, PartialEq, Props)]
pub struct IconProps {
    #[props(default)]
    pub icon: MdiIcon,
    #[props(default)]
    pub class: String,
    #[props(default = 24)]
    pub size: u32,
    #[props(default)]
    pub color: String,
}

#[cfg(feature = "tairitsu")]
impl Default for IconProps {
    fn default() -> Self {
        Self {
            icon: MdiIcon::Help,
            class: String::new(),
            size: 24,
            color: String::new(),
        }
    }
}
```

**之后（11行）：**
```rust
#[cfg(feature = "tairitsu")]
#[define_props]
pub struct IconProps {
    #[default(MdiIcon::Help)]
    pub icon: MdiIcon,
    pub class: String,           // 自动 String::new()
    #[default(24)]
    pub size: u32,
    pub color: String,           // 自动 String::new()
}
```

**减少约 60%**

## 验收标准 ✅

- [x] 宏编译通过
- [x] 生成的 Props 结构体与手动编写的功能等价
- [x] hikari-icons 中 IconProps 使用新宏
- [ ] hikari-components 中其他组件使用新宏（待推广）

## 后续优化

1. 将更多 hikari-components 的 Props 定义迁移到 `#[define_props]`
2. 考虑添加 `#[optional]` 属性作为 `Option<T>` 的语法糖
3. 考虑添加 `#[into]` 属性自动添加 `#[props(into)]`
