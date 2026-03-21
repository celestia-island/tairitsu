# Browser Glue TypeScript 错误修复计划

## 当前状态

**错误数量：0 个** ✅ (从原始 ~2500+ 减少至 0)

## 已完成

### 类型映射修复
- [x] 修复 `type_mapper.py` 缓存 bug - 缓存使用 `id(wit_type)` 导致类型映射错误

### WIT 定义修复
- [x] `events.wit` - 修复 `clipboard-change-event.get-types` 返回类型 (`list<string>` → `string`)
- [x] `performance.wit` - 修复 `measure()` 函数签名 (添加缺失的参数名 `end-mark`)
- [x] `performance.wit` - 移除不存在的 `get-measure-name` 函数
- [x] `url.wit` - 修复 `URL.href` 类型 (`u64` → `string`)

### 配置项修复
- [x] 移除 `ENUM_PROPERTIES` 中的 `("url", "href")` - href 是 string 不是 enum
- [x] 移除 `ENUM_PROPERTIES` 中的 `("performance-navigation", "redirectCount")` - 是 number 不是 enum
- [x] 添加 `NUMBER_TO_BIGINT_PROPERTIES` 条目:
  - `("performance-navigation-timing", "redirectCount")`
  - `("performance-navigation", "redirectCount")`
  - `("performance-resource-timing", "redirectStart")`
  - `("performance-resource-timing", "responseStatus")`
- [x] 添加 `DICTIONARY_PARAMETER_TYPES` 条目:
  - `("performance", "measure", "start-or-measure-options")`
  - `("performance", "clear-measures", "measure-name")`

### 配置项添加 (历史)
- [x] `SYNTHETIC_HANDLE_TYPES` - 为返回对象的类型创建 handle 表
- [x] `HANDLE_RETURNING_FUNCTIONS` - 返回对象需要 wrap 的方法
- [x] `HANDLE_RETURNING_ARRAY_PROPERTIES` - 返回对象数组的方法
- [x] `PARAMETER_BIGINT_TO_NUMBER` - 参数类型转换映射
- [x] `PARAMETER_HANDLE_MAPPING` - 参数 handle lookup 映射
- [x] `BOOLEAN_TO_BIGINT_PROPERTIES` - boolean→bigint 的属性
- [x] `ENUM_VALUE_MAPPINGS` - 枚举值映射
- [x] `ENUM_SETTER_PROPERTIES` - bigint→string 枚举的 setter
- [x] `PROPERTIES_NEEDING_TYPE_ASSERTION` - 需要 `as any` 的属性
- [x] `METHODS_NEEDING_TYPE_ASSERTION` - 需要 `as any` 的方法
- [x] `ASYNC_METHOD_OVERRIDES` - 异步方法标记
- [x] `GETTER_BUT_ACTUALLY_METHOD` - getter 实际是方法
- [x] `SETTER_BUT_ACTUALLY_METHOD` - setter 实际是方法
- [x] `READONLY_ARRAY_PROPERTIES` - 只读数组属性
- [x] `CUSTOM_TYPE_DEFINITIONS` - 自定义类型定义
- [x] `GETTER_RETURN_COALESCING` - getter 返回值合并
- [x] `GETTER_HANDLE_NON_NULL_ASSERTION` - getter handle 非空断言

### 代码生成器修复 (历史)
- [x] 修复重复 return 语句 (NUMBER_TO_BIGINT_PROPERTIES)
- [x] 添加 `optional-handle-strict` 转换类型
- [x] 添加 `boolean-or-false` 转换类型
- [x] 添加 `spread-handle-array` 转换类型
- [x] 添加 `handle-array` 转换类型
- [x] 添加 event-handler 类型处理

### 错误类型修复 (历史)
- [x] 修复 TS2304 (Cannot find name)
- [x] 修复 TS2322 (Type mismatch)
- [x] 修复 TS2339 (Property not exist)
- [x] 修复 TS2345 (Parameter type mismatch)
- [x] 修复 TS2551 (Method name mismatch)
- [x] 修复 TS2552 (Cannot find name)
- [x] 修复 TS2554 (Argument count mismatch)
- [x] 修复 TS2367 (Type comparison)
- [x] 修复 TS2393 (Duplicate function)
- [x] 修复 TS2678 (Type comparison)
- [x] 修复 TS2769 (No overload matches)

## 关键文件

- `scripts/generator/config.py` - 主配置文件
- `scripts/generator/code_gen.py` - TypeScript 代码生成逻辑
- `scripts/type_mapper.py` - WIT 类型到 TypeScript 类型映射
- `packages/browser-worlds/wit/generated/*.wit` - WIT 定义文件

## 命令

```bash
# 重新生成
python3 scripts/generate_browser_glue.py

# 验证
cd packages/browser-glue && npx tsc --noEmit

# 构建
cd packages/browser-glue && npm run build
```
