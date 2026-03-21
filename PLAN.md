# Browser Glue TypeScript 错误修复计划

## 当前状态

**错误数量：171 个** (从原始 ~2500+ 减少约 93%)

## 错误分布

| 错误类型 | 数量 | 描述 |
|---------|------|------|
| TS2322 | 125 | 类型不匹配 |
| TS2345 | 39 | 参数类型不匹配 |
| TS2559 | 2 | 函数签名不匹配 |
| 其他 | 5 | 其他错误 |

## 已完成

- [x] 修复 `Cannot find name 'entry'` (TS2304)
- [x] 修复 `Parameter 'result' implicitly has an 'any' type` (TS7006)
- [x] 添加 `ASYNC_METHOD_OVERRIDES` 配置
- [x] 修复 `browser_attr` 映射
- [x] 添加 `ENUM_PROPERTIES` 和 `ENUM_VALUE_MAPPINGS`
- [x] 添加 `NUMBER_TO_BIGINT_PROPERTIES`
- [x] 添加 `BOOLEAN_TO_BIGINT_PROPERTIES`
- [x] 添加 `CUSTOM_TYPE_DEFINITIONS`
- [x] 添加 `GETTER_BUT_ACTUALLY_METHOD`
- [x] 添加 `PROPERTIES_NEEDING_TYPE_ASSERTION`
- [x] 添加 `SYNTHETIC_HANDLE_TYPES` 用于所有返回对象的类型
- [x] 添加 `HANDLE_RETURNING_FUNCTIONS` 用于返回对象的方法
- [x] 添加 `PARAMETER_BIGINT_TO_NUMBER` 用于参数转换
- [x] 添加 `DICTIONARY_PARAMETER_TYPES` 用于字典参数
- [x] 添加 `ENUM_SETTER_PROPERTIES` 用于 setter 枚举转换
- [x] 修复 TS2300, TS2304, TS2339, TS2349, TS2551, TS2552, TS2554, TS2559, TS2693, TS2678, TS2769, TS18046, TS2355, TS4104 等错误类型

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

- `78a752a` - 初始修复
- `4861ade` - 消除 TS2304 错误
- `89a7259` - 消除 TS2693 和 TS2339 错误
- `5a773cc` - 消除 TS2393 重复函数错误
- `01d070c` - 减少 TS2322 和 TS2345 错误
- `3665359` - 消除 TS18046 错误
- `d118d5f` - 减少 TS2322、TS2345、TS2554、TS2551 错误
- `39da64d` - 消除 TS2300、TS2304、TS2349 错误
- `c4c1b75` - 消除 TS2769 错误
- `de571f8` - 消除 TS2304 和 TS4104 错误
- `eb23c03` - 消除 TS2554 和 TS2355 错误
- `f2d2d10` - 消除 TS2304 错误
- `6ef7239` - 消除 TS2552 和 TS2339 错误
- `0a4b683` - 减少 TS2322、TS2345 和其他错误
- `af517d1` - 消除 TS2339 错误
- `432d6d7` - 消除 TS2552 错误
- `aca5733` - 继续减少 TS2322 和 TS2345 错误
