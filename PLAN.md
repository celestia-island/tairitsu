# 汇总：packages/browser-glue 生成文件的 TypeScript 错误

说明：下面按 TypeScript 错误代码归并，同类错误合并列出受影响的文件和行号（以文件->行号序列表示）。每一类错误下先给出简要含义说明，再列出文件与行号供定位。该错误集合来自编辑器/编译器诊断（生成文件位于 packages/browser-glue/src/generated）。

## 已修复的问题

1. **`Cannot find name 'entry'`** (TS2304)
   - 原因：`_render_async_body` 中 `.then()` 回调使用了 `entry` 变量但没有定义
   - 修复：在 `.then()` 开头添加 `const entry = _asyncHandles.get(requestId);`

2. **`Parameter 'result' implicitly has an 'any' type`** (TS7006)
   - 原因：async 函数中 `.then()` 回调的 `result` 参数缺少类型注解
   - 修复：将 `.then((result) =>` 改为 `.then((result: unknown) =>`

3. **部分 async 方法错误标记**
   - 原因：`close`, `reset`, `abort` 等方法在某些接口上不是 async 的
   - 修复：添加 `ASYNC_METHOD_OVERRIDES` 配置来精确控制每个接口的 async 方法

4. **`browser_attr` 未使用 API 名称映射**
   - 原因：`browser_attr` 只使用 `kebab_to_camel`，没有使用 `BROWSER_API_NAME_MAPPINGS`
   - 修复：在 `generate_function` 中使用 `BROWSER_API_NAME_MAPPINGS.get(attr_name, ...)`

5. **枚举属性返回 string 而非 bigint**
   - 原因：某些属性在浏览器中是 string 枚举，但 WIT 定义为 bigint
   - 修复：添加 `ENUM_PROPERTIES` 和 `ENUM_VALUE_MAPPINGS` 配置，生成 switch 语句转换

6. **number 属性返回 number 而非 bigint**
   - 原因：某些属性在浏览器中是 number，但 WIT 定义为 bigint
   - 修复：添加 `NUMBER_TO_BIGINT_PROPERTIES` 配置，生成 `BigInt()` 转换

7. **参数 handle lookup 目标接口错误**
   - 原因：`PARAMETER_HANDLE_MAPPING` 中的目标接口设置不正确
   - 修复：更新映射表使用正确的接口名

## 剩余问题（约 2478 个错误）

当前错误分布：
- TS2322 (822个): 类型不匹配
- TS2345 (311个): 参数类型不匹配
- TS2339 (213个): 属性不存在
- TS2304 (139个): 未声明的标识符
- TS2551 (69个): 方法名错误
- TS2769 (67个): 没有匹配的重载

### 主要问题类别

1) 错误码 2322 — "Type 'A' is not assignable to type 'B'"
   - 含义：类型不匹配，赋值或返回值的类型与目标类型不兼容
   - 常见原因：
     - getter 返回对象而非 bigint handle
     - 方法返回 Promise 但被当作普通方法处理
     - 返回类型应该是 number/string 但定义为 bigint

2) 错误码 2345 — "Argument of type 'A' is not assignable to parameter of type 'B'"
   - 含义：函数调用/方法调用时传参类型不匹配
   - 常见原因：
     - 参数需要 handle lookup 但没有配置
     - 参数需要类型转换（如 bigint -> number）

3) 错误码 2339 — "Property 'X' does not exist on type 'Y'"
   - 含义：访问对象上不存在的属性或方法
   - 常见原因：
     - 某些属性在 TypeScript DOM 类型中不存在
     - 属性名拼写错误

4) 错误码 2304 — "Cannot find name 'X'"
   - 含义：引用了未声明的变量/标识符
   - 常见原因：
     - 缺失类型定义（如 WebGLObject）

## 下一步建议

1. **添加更多返回类型转换配置**
   - 扩展 `ENUM_PROPERTIES` 和 `NUMBER_TO_BIGINT_PROPERTIES`
   - 添加对象到 handle 的转换逻辑

2. **添加更多参数类型转换配置**
   - 扩展 `PARAMETER_HANDLE_MAPPING`
   - 扩展 `DICTIONARY_PARAMETER_TYPES`
   - 扩展 `PARAMETER_BIGINT_TO_NUMBER`

3. **添加缺失的类型定义**
   - 在 `CUSTOM_TYPE_DEFINITIONS` 中添加缺失的类型

4. **修复属性名错误**
   - 扩展 `BROWSER_API_NAME_MAPPINGS`

—— 结束 ——
