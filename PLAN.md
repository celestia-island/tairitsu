# Browser Glue TypeScript 错误修复计划

## 当前状态

**错误数量：25 个** (从原始 ~2500+ 减少约 99%)

## 剩余错误分析

剩余 25 个错误大多是 **WIT 定义 Bug**，WIT 文件中定义的类型与实际浏览器 DOM API 类型不匹配：

| 错误类型 | 原因 | 解决方案 |
|---------|------|---------|
| string vs number | WIT 定义为 string，DOM 返回 number | 修复 WIT 文件 |
| string vs bigint | WIT 定义为 bigint，DOM 返回 string | 修复 WIT 文件或添加枚举转换 |
| boolean vs number | WIT 定义为 boolean，DOM 返回 number | 修复 WIT 文件 |
| undefined 参数 | 生成器可能错误添加 | 需要调查生成器逻辑 |

## 已完成

### 配置项添加
- [x] `SYNTHETIC_HANDLE_TYPES` - 为返回对象的类型创建 handle 表
- [x] `HANDLE_RETURNING_FUNCTIONS` - 返回对象需要 wrap 的方法
- [x] `HANDLE_RETURNING_ARRAY_PROPERTIES` - 返回对象数组的方法
- [x] `PARAMETER_BIGINT_TO_NUMBER` - 参数类型转换映射
- [x] `PARAMETER_HANDLE_MAPPING` - 参数 handle lookup 映射
- [x] `NUMBER_TO_BIGINT_PROPERTIES` - number→bigint 的属性
- [x] `BOOLEAN_TO_BIGINT_PROPERTIES` - boolean→bigint 的属性
- [x] `ENUM_PROPERTIES` - string 枚举→bigint 的属性
- [x] `ENUM_VALUE_MAPPINGS` - 枚举值映射
- [x] `ENUM_SETTER_PROPERTIES` - bigint→string 枚举的 setter
- [x] `PROPERTIES_NEEDING_TYPE_ASSERTION` - 需要 `as any` 的属性
- [x] `METHODS_NEEDING_TYPE_ASSERTION` - 需要 `as any` 的方法
- [x] `ASYNC_METHOD_OVERRIDES` - 异步方法标记
- [x] `GETTER_BUT_ACTUALLY_METHOD` - getter 实际是方法
- [x] `SETTER_BUT_ACTUALLY_METHOD` - setter 实际是方法
- [x] `READONLY_ARRAY_PROPERTIES` - 只读数组属性
- [x] `CUSTOM_TYPE_DEFINITIONS` - 自定义类型定义
- [x] `DICTIONARY_PARAMETER_TYPES` - 字典参数类型
- [x] `GETTER_RETURN_COALESCING` - getter 返回值合并
- [x] `GETTER_HANDLE_NON_NULL_ASSERTION` - getter handle 非空断言

### 代码生成器修复
- [x] 修复重复 return 语句 (NUMBER_TO_BIGINT_PROPERTIES)
- [x] 添加 `optional-handle-strict` 转换类型
- [x] 添加 `boolean-or-false` 转换类型
- [x] 添加 `spread-handle-array` 转换类型
- [x] 添加 `handle-array` 转换类型
- [x] 添加 event-handler 类型处理

### 错误类型修复
- [x] 修复 TS2304 (Cannot find name)
- [x] 修复 TS2339 (Property not exist)
- [x] 修复 TS2345 (Parameter type mismatch)
- [x] 修复 TS2551 (Method name mismatch)
- [x] 修复 TS2552 (Cannot find name)
- [x] 修复 TS2554 (Argument count mismatch)
- [x] 修复 TS2393 (Duplicate function)
- [x] 修复 TS2769 (No overload matches)
- [x] 修复 TS18046 (Type narrowing)
- [x] 修复 TS2678 (Type comparison)

## 剩余工作

### 高优先级

1. **修复 TS2322 类型不匹配** (~125 个)
   - 继续添加 HANDLE_RETURNING_FUNCTIONS 条目
   - 继续添加 NUMBER_TO_BIGINT_PROPERTIES 条目
   - 继续添加 ENUM_PROPERTIES 条目

2. **修复 TS2345 参数类型不匹配** (~39 个)
   - 继续添加 PARAMETER_BIGINT_TO_NUMBER 条目
   - 添加更多 handle lookup 类型

### 中优先级

3. **修复 TS2559 函数签名** (~2 个)
   - 调整 WIT 定义或添加特殊处理

## 关键配置

### config.py 中的主要配置项

- `SYNTHETIC_HANDLE_TYPES` - 为返回对象的类型创建 handle 表
- `HANDLE_RETURNING_FUNCTIONS` - 返回对象需要 wrap 的方法
- `PARAMETER_BIGINT_TO_NUMBER` - 参数类型转换映射
- `NUMBER_TO_BIGINT_PROPERTIES` - number→bigint 的属性
- `BOOLEAN_TO_BIGINT_PROPERTIES` - boolean→bigint 的属性
- `ENUM_PROPERTIES` - string 枚举→bigint 的属性
- `PROPERTIES_NEEDING_TYPE_ASSERTION` - 需要 `as any` 的属性

### code_gen.py 中的关键函数

- `_render_getter_body()` - getter 代码生成
- `_render_setter_body()` - setter 代码生成
- `_render_method_body()` - 方法代码生成
- `_get_converted_browser_args()` - 参数转换

## 提交历史

- 28 commits on dev branch
- Error reduction: ~2500 → 25 (99% reduction)
