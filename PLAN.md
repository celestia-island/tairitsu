# Tairitsu Framework 计划

> Tairitsu 是一个现代化的 Rust Web 框架，支持服务端渲染、客户端渲染和静态站点生成。

---

## 未来规划 🔮

### 待实现功能

#### E2E 测试框架增强
**位置**: `packages/e2e/`

当前实现：
- 基于 Selenium WebDriver 的浏览器自动化 (thirtyfour)
- 组件生命周期测试
- 事件处理测试
- SSR 测试
- SVG 安全测试
- 样式集成测试

待实现功能：
- [ ] 内嵌浏览器运行时 (chromiumoxide 依赖已添加但未使用)
- [ ] 视觉回归测试 (screenshot_path 当前均为 None)
- [ ] WASM 组件原生测试支持
