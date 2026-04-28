# 故障排除指南

使用 Tairitsu browser-glue 和 Component Model 时的常见问题与解决方案。

## 构建错误

### 找不到 wasm32-wasip2 目标

**错误：**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**解决方案：**
```bash
rustup target add wasm32-wasip2
```

### wit-bindgen 版本不匹配

**错误：**
```
error: failed to select a version for `wit-bindgen`
```

**解决方案：**
确保 `Cargo.toml` 中的 `wit-bindgen` 版本匹配：
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### TypeScript 编译错误

**错误：**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**解决方案：**
重新生成 glue 并重新构建：
```bash
cd packages/browser-glue
npm run build
```

## 运行时错误

### 缺少宿主导入

**错误：**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**解决方案：**
1. 确保 import map 已配置：
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. 验证 browser-glue 文件存在于输出目录中。

### 组件初始化失败

**错误：**
```
Error: Component instantiation failed: undefined import
```

**解决方案：**
检查所有需要的 WIT 导入在 browser-glue 中都有对应的实现。

### jco 转译错误

**错误：**
```
Error: Failed to transpile component
```

**解决方案：**
1. 确保 jco 已安装：
```bash
npm install -g @bytecodealliance/jco
```

2. 验证 WASM 组件有效：
```bash
wasm-tools print component.wasm
```

## 调试技巧

### 启用调试日志

在浏览器控制台中：
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### 检查 WIT 绑定

查看生成的绑定：
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### 浏览器开发者工具

1. 打开开发者工具（F12）
2. 检查控制台中的错误
3. 使用网络标签查看模块加载失败
4. 使用源代码标签进行调试

### 组件验证

```bash
# 验证组件结构
wasm-tools validate component.wasm

# 打印组件内容
wasm-tools print component.wasm
```

## 常见问题

### 句柄未找到

**症状：** DOM 操作返回 `null`

**原因：** 句柄被垃圾回收或未注册

**解决方案：** 确保元素在 JavaScript 中保持引用

### 事件未触发

**症状：** 事件处理程序未被调用

**原因：** 监听器 ID 不匹配或事件类型错误

**解决方案：** 检查 `addEventListener` 返回有效的监听器 ID

### 内存泄漏

**症状：** 内存使用随时间增加

**原因：** 句柄在使用后未释放

**解决方案：** 对象使用完毕后调用 `dropHandle()`

## 性能问题

### 组件加载缓慢

**解决方案：**
1. 使用 release 构建：`cargo build --release`
2. 在 `Cargo.toml` 中启用 LTO：
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### 事件延迟高

**解决方案：**
1. 避免在处理程序中使用同步操作
2. 使用 `requestAnimationFrame` 进行视觉更新
3. 对频繁事件进行防抖处理

## 获取帮助

1. 查看现有问题：https://github.com/anomalyco/opencode/issues
2. 查阅 `docs/` 目录中的文档
3. 研究 `examples/website/` 中的示例代码
