# Tairitsu Type-Safe Command Architecture

本文档详细说明 Tairitsu 框架中基于枚举的类型安全命令架构，实现编译时类型检查的双向通信。

## 1. 架构概述

### 1.1 设计目标

- **编译时类型安全**：所有命令和参数都通过枚举而非字符串表示
- **强类型约束**：使用 Rust 的类型系统确保正确性
- **序列化透明**：通过 serde 自动序列化/反序列化
- **向后兼容**：支持旧的字符串命令作为 fallback

### 1.2 核心组件

```
┌─────────────────────────────────────────────────────────┐
│                    Tairitsu Framework                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Host Side                       Guest Side (WASM)      │
│  ┌──────────────┐                ┌──────────────┐      │
│  │HostCommands  │◄───JSON RPC───►│HostCommands  │      │
│  │  - GetInfo   │                │  - GetInfo   │      │
│  │  - Echo      │                │  - Echo      │      │
│  │  - Custom    │                │  - Custom    │      │
│  └──────────────┘                └──────────────┘      │
│        │                                  │             │
│        │                                  │             │
│  ┌──────────────┐                ┌──────────────┐      │
│  │GuestCommands │◄───JSON RPC───►│GuestCommands │      │
│  │  - Greet     │                │  - Greet     │      │
│  │  - Compute   │                │  - Compute   │      │
│  │  - CallHost  │                │  - CallHost  │      │
│  │  - Custom    │                │  - Custom    │      │
│  └──────────────┘                └──────────────┘      │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## 2. 命令枚举定义

### 2.1 Host Commands (Guest → Host)

**位置**: `packages/vm/src/commands.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum HostCommands {
    /// 获取宿主信息
    GetInfo,
    
    /// 回显消息
    Echo(String),
    
    /// 自定义命令（向后兼容）
    Custom { name: String, data: String },
}
```

**特点**：
- 使用 `#[serde(tag = "type", content = "data")]` 实现标签化的 JSON 序列化
- `GetInfo` 无参数
- `Echo` 接受单个字符串参数
- `Custom` 作为扩展点，支持任意自定义命令

**JSON 序列化示例**：
```json
// GetInfo
{"type":"GetInfo"}

// Echo
{"type":"Echo","data":"Hello"}

// Custom
{"type":"Custom","data":{"name":"custom_cmd","data":"some_data"}}
```

### 2.2 Guest Commands (Host → Guest)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GuestCommands {
    /// 问候消息
    Greet(String),
    
    /// 执行计算
    Compute(String),
    
    /// 回调宿主
    CallHost(String),
    
    /// 自定义命令
    Custom { name: String, data: String },
}
```

### 2.3 Response Types

```rust
// Host 响应类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HostResponse {
    Info {
        name: String,
        version: String,
        status: String,
    },
    Text(String),
}

// Guest 响应类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GuestResponse {
    Text(String),
}
```

### 2.4 Log Levels

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
```

## 3. 类型安全的 API

### 3.1 Host 端 API

```rust
// 创建 Container 并设置类型安全的处理器
let mut container = Container::new(&image)?;

// 类型安全的 execute 处理器
container.on_execute(|command: HostCommands| -> Result<HostResponse, String> {
    match command {
        HostCommands::GetInfo => Ok(HostResponse::Info {
            name: "Tairitsu Host".to_string(),
            version: "0.1.0".to_string(),
            status: "running".to_string(),
        }),
        HostCommands::Echo(msg) => Ok(HostResponse::Text(msg)),
        HostCommands::Custom { name, data } => {
            Ok(HostResponse::Text(format!("Custom: {} - {}", name, data)))
        }
    }
});

// 类型安全的 log 处理器
container.on_log(|level: LogLevel, message: String| {
    println!("[Guest][{}] {}", level, message);
});

// 发送类型安全的命令到 Guest
let response = container.send_command(GuestCommands::Greet("Hello".to_string()))?;
```

### 3.2 Guest 端 API

```rust
// Guest 端接收并处理类型化的命令
fn handle_command(command: String, _payload: String) -> Result<String, String> {
    // 反序列化为类型化的命令
    let cmd: GuestCommands = serde_json::from_str(&command)?;
    
    let response = match cmd {
        GuestCommands::Greet(msg) => {
            GuestResponse::Text(format!("Hello! You said: {}", msg))
        }
        GuestCommands::Compute(data) => {
            let result = data.len() * 42;
            GuestResponse::Text(format!("Result: {}", result))
        }
        GuestCommands::CallHost(payload) => {
            // 发送类型化的命令到 Host
            let host_cmd = HostCommands::Echo(payload);
            let cmd_json = serde_json::to_string(&host_cmd)?;
            
            let response_json = tairitsu::core::host_api::execute(&cmd_json, "")?;
            let host_response: HostResponse = serde_json::from_str(&response_json)?;
            
            match host_response {
                HostResponse::Text(text) => GuestResponse::Text(format!("Host: {}", text)),
                HostResponse::Info { name, .. } => GuestResponse::Text(name),
            }
        }
        GuestCommands::Custom { name, data } => {
            GuestResponse::Text(format!("Custom: {} - {}", name, data))
        }
    };
    
    serde_json::to_string(&response)
}
```

## 4. 运行时验证

### 4.1 构建和运行

```bash
# 构建 WASM Guest
cargo build --target wasm32-wasip1 --release --package tairitsu-example-hybrid

# 运行 Host
cargo run --package tairitsu-example-hybrid --bin host --features host-binary --release
```

### 4.2 运行日志（类型安全版本）

```
=== Tairitsu Hybrid Example - Host Side (Type-Safe Commands) ===

Loading WASM module from: .../target/wasm32-wasip1/release/tairitsu_example_hybrid.wasm
WASM module loaded (203259 bytes)

Registering WASM module as image 'hybrid-example:latest'
Creating container from image...

=== Initializing Guest Module ===
[Guest Log][INFO] Guest module initialized
[Host] Received execute request: GetInfo                    ← 类型安全的枚举
[Guest Log][INFO] Host info: Tairitsu Host v0.1.0 (running)

=== Sending Typed Commands to Guest ===

[Host] Sending typed command: Greet("Tairitsu Framework")   ← 类型安全的枚举
[Guest Log][INFO] Received typed command: Greet("Tairitsu Framework")
[Host] Guest response: Text("Hello from WASM guest! You said: Tairitsu Framework")

[Host] Sending typed command: Compute("Hello World")        ← 类型安全的枚举
[Guest Log][INFO] Received typed command: Compute("Hello World")
[Host] Guest response: Text("Computed result: 462")

[Host] Sending typed command: CallHost("This message goes to host and back")  ← 嵌套调用
[Guest Log][INFO] Received typed command: CallHost("This message goes to host and back")
[Host] Received execute request: Echo("This message goes to host and back")
[Host] Guest response: Text("Host echoed: This message goes to host and back")

=== Example Complete ===
```

## 5. 类型安全的优势

### 5.1 编译时检查

```rust
// ✅ 编译通过 - 正确的类型
container.send_command(GuestCommands::Greet("Hello".to_string()))?;

// ❌ 编译错误 - 类型不匹配
container.send_command("greet")?;  // Error: expected GuestCommands, found &str

// ❌ 编译错误 - 缺少参数
container.send_command(GuestCommands::Greet())?;  // Error: missing argument
```

### 5.2 IDE 支持

- **自动补全**：IDE 可以提示所有可用的命令枚举值
- **类型提示**：显示每个命令需要的参数类型
- **重构安全**：重命名枚举值会自动更新所有使用位置
- **文档集成**：枚举的文档注释在 IDE 中可见

### 5.3 运行时安全

- **反序列化验证**：JSON 格式错误会在反序列化时被捕获
- **模式匹配完整性**：Rust 编译器确保所有枚举分支都被处理
- **类型转换安全**：serde 确保 JSON 和 Rust 类型一致

## 6. 扩展机制

### 6.1 添加新命令

```rust
// 1. 在 commands.rs 中扩展枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GuestCommands {
    Greet(String),
    Compute(String),
    CallHost(String),
    
    // 新增命令
    ProcessData { input: Vec<u8>, format: String },
    
    Custom { name: String, data: String },
}

// 2. 在 Guest 中处理新命令
match cmd {
    GuestCommands::ProcessData { input, format } => {
        // 处理新命令
        let processed = process(input, &format)?;
        GuestResponse::Text(processed)
    }
    // ... 其他分支
}

// 3. 在 Host 中使用
container.send_command(GuestCommands::ProcessData {
    input: vec![1, 2, 3],
    format: "hex".to_string(),
})?;
```

### 6.2 自定义命令扩展

对于需要动态扩展的场景，使用 `Custom` 变体：

```rust
// Host 发送自定义命令
container.send_command(GuestCommands::Custom {
    name: "my_plugin".to_string(),
    data: serde_json::to_string(&my_custom_data)?,
})?;

// Guest 处理自定义命令
GuestCommands::Custom { name, data } => {
    if name == "my_plugin" {
        let custom_data: MyCustomType = serde_json::from_str(&data)?;
        // 处理自定义数据
    }
    // ...
}
```

## 7. 性能考虑

### 7.1 序列化开销

- 使用 `serde_json` 进行序列化，性能优秀
- 对于高频调用，可以考虑使用二进制序列化（如 bincode）
- 命令枚举本身零开销（编译时解析）

### 7.2 向后兼容

框架保留了对旧字符串命令的支持（通过 `Custom` 变体），确保平滑迁移。

## 8. 最佳实践

### 8.1 命令设计原则

1. **保持枚举简洁**：每个命令应该有明确的职责
2. **使用结构化数据**：复杂参数使用结构体而非多个独立字段
3. **文档化命令**：为每个枚举值添加文档注释
4. **版本化**：考虑在命令中包含版本信息

### 8.2 错误处理

```rust
// 使用 Result 类型传递错误信息
container.on_execute(|command: HostCommands| -> Result<HostResponse, String> {
    match command {
        HostCommands::GetInfo => {
            // 成功路径
            Ok(HostResponse::Info { ... })
        }
        HostCommands::Echo(msg) => {
            if msg.is_empty() {
                // 错误路径
                Err("Empty message not allowed".to_string())
            } else {
                Ok(HostResponse::Text(msg))
            }
        }
    }
});
```

## 9. 总结

Tairitsu 的类型安全命令架构提供了：

✅ **编译时类型检查** - 消除运行时类型错误
✅ **强类型约束** - 利用 Rust 类型系统确保正确性  
✅ **IDE 友好** - 完整的自动补全和类型提示
✅ **可扩展** - 通过枚举添加新命令，通过 Custom 支持动态扩展
✅ **向后兼容** - 支持旧的字符串命令
✅ **高性能** - 零开销抽象 + 高效序列化

这种架构将动态的跨语言通信转变为静态类型检查的过程，大大提高了代码的可维护性和可靠性。
