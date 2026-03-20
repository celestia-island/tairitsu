# Browser Glue TypeScript 错误修复计划

## 当前状态

**错误数量：1778 个** (从原始 ~2500+ 减少)

## 错误分布

| 错误类型 | 数量 | 描述 |
|---------|------|------|
| TS2322 | 806 | 类型不匹配 - 返回对象/number/string 但期望 bigint |
| TS2345 | 267 | 参数类型不匹配 |
| TS2304 | 246 | 未声明的标识符 (`_nextWebGlObject`, `getU64` 等) |
| TS2693 | 118 | 类作为值使用 |
| TS18046 | 95 | 类型缩小问题 |
| TS2769 | 68 | 没有匹配的重载 |
| TS2339 | 58 | 属性不存在 |

## 根本原因分析

### 1. 缺少 Handle 表和辅助函数 (TS2304)
- 生成代码引用 `_nextWebGlObject`, `_nextString` 等变量
- 但这些只在接口有 `handle_type` 时才生成
- 需要为返回对象的方法创建对应的 handle 表

### 2. 类型不匹配 (TS2322)
- WIT 定义所有返回值为 bigint (handle)
- 但浏览器 API 返回对象、number、string、boolean
- 需要在 `HANDLE_RETURNING_FUNCTIONS` 中配置 wrap 逻辑

### 3. 参数类型不匹配 (TS2345)
- bigint 参数需要转换为正确的浏览器类型
- 需要扩展 `PARAMETER_BIGINT_TO_NUMBER` 和 `PARAMETER_HANDLE_MAPPING`

## 修复策略

### 阶段 1: 修复 TS2304 缺失标识符 (高优先级)
- 为返回对象的方法添加 handle 表
- 添加缺失的辅助函数 (`getU64`, `getOption`, `u64`)
- 预期减少 ~200 错误

### 阶段 2: 修复 TS2322 类型不匹配
- 扩展 `HANDLE_RETURNING_FUNCTIONS` 配置
- 添加更多 `NUMBER_TO_BIGINT_PROPERTIES`
- 添加更多 `BOOLEAN_TO_BIGINT_PROPERTIES`
- 添加更多 `ENUM_PROPERTIES`
- 预期减少 ~500 错误

### 阶段 3: 修复 TS2345 参数类型
- 扩展 `PARAMETER_BIGINT_TO_NUMBER`
- 扩展 `PARAMETER_HANDLE_MAPPING`
- 预期减少 ~200 错误

### 阶段 4: 修复剩余错误
- TS2693 类作为值
- TS2339 属性不存在
- TS18046 类型缩小

## 已完成

- [x] 修复 `Cannot find name 'entry'`
- [x] 修复 `Parameter 'result' implicitly has an 'any' type`
- [x] 添加 `ASYNC_METHOD_OVERRIDES` 配置
- [x] 修复 `browser_attr` 映射
- [x] 添加 `ENUM_PROPERTIES` 和 `ENUM_VALUE_MAPPINGS`
- [x] 添加 `NUMBER_TO_BIGINT_PROPERTIES`
- [x] 添加 `BOOLEAN_TO_BIGINT_PROPERTIES`
- [x] 添加 `CUSTOM_TYPE_DEFINITIONS`
- [x] 添加 `GETTER_BUT_ACTUALLY_METHOD`
- [x] 添加 `PROPERTIES_NEEDING_TYPE_ASSERTION`
- [x] 提交: `78a752a` - "fix: reduce TypeScript errors in browser-glue generated code"

## 进行中

- [ ] 阶段 1: 修复 TS2304 缺失标识符
- [ ] 阶段 2: 修复 TS2322 类型不匹配
- [ ] 阶段 3: 修复 TS2345 参数类型
- [ ] 阶段 4: 修复剩余错误

## 目标

- 所有 TypeScript 编译错误归零
- 无假实现、TODO 或 Mock 接口
- E2E 测试通过
