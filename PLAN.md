# Tairitsu 开发计划

## 来自 hikari-animation 的功能需求

### 已完成

- [x] `DomRect` 添加 `Clone/Copy` derive
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub struct DomRect {
      pub x: f64,
      pub y: f64,
      pub width: f64,
      pub height: f64,
  }
  ```

### 待添加的 WIT 接口支持

以下功能需要添加到 tairitsu WIT 接口中，以支持完整的动画系统功能：

#### 1. prefers-reduced-motion 检测

需要 `window.matchMedia()` API 支持：

```wit
/// 检测用户是否偏好减少动画
match-media: func(query: string) -> bool
```

#### 2. MediaQueryList 事件监听

需要 `MediaQueryList.addEventListener()` 支持，用于监听媒体查询变化：

```wit
/// 媒体查询列表监听器
interface media-query-list {
    /// 添加媒体查询变化监听器
    add-change-listener: func(callback: func()) -> listener
    /// 移除监听器
    remove-listener: func(listener: listener)
}
```

### 优先级

- **高优先级**: `matchMedia` API - 许多无障碍功能依赖此功能
- **中优先级**: `MediaQueryList` 事件监听 - 动态响应系统偏好设置变化

---

## 其他已知需求

(待补充...)
