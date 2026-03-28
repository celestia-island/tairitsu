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

- [x] `matchMedia` API 支持
  - 添加 `Platform::match_media()` 方法
  - 添加 `Platform::media_query_list_get_media()` 方法
  - 添加 `Platform::media_query_list_get_matches()` 方法
  - 在 WIT 接口中定义 `media-query-list-callbacks` 回调接口
  - 在 browser-glue 中实现 MediaQueryList 事件监听机制

- [x] MediaQueryList 事件监听
  - 添加 `Platform::media_query_list_add_listener()` 方法
  - 添加 `Platform::media_query_list_remove_listener()` 方法
  - 实现媒体查询变化的回调机制

---

## 其他已知需求

(待补充...)
