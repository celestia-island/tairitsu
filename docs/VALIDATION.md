# Tairitsu Framework Validation Report

本文档验证 Tairitsu 框架的双向通信功能，包括源代码分析和运行时日志证明。

## 1. WIT 接口定义

### 位置：`wit/world.wit`

```wit
package tairitsu:core@0.1.0;

/// Guest interface - functions that the guest (WASM module) exports and host calls
interface guest-api {
    /// Initialize the guest module
    init: func() -> result<_, string>;
    
    /// Handle a command from the host
    handle-command: func(command: string, payload: string) -> result<string, string>;
}

/// Host interface - functions that the host provides and guest can call
interface host-api {
    /// Execute a command on the host
    execute: func(command: string, payload: string) -> result<string, string>;
    
    /// Log a message to the host
    log: func(level: string, message: string);
}

/// The main world for Tairitsu components
world tairitsu {
    /// Guest imports the host API
    import host-api;
    
    /// Guest exports its API
    export guest-api;
}
```

**说明**：
- `guest-api`: Guest (WASM) 导出的接口，Host 可以调用
- `host-api`: Host 提供的接口，Guest 可以调用
- 通过 WIT Component Model 实现类型安全的双向通信

## 2. Guest 端实现（WASM）

### 位置：`examples/hybrid/src/lib.rs`

```rust
#[cfg(target_family = "wasm")]
wit_bindgen::generate!({
    path: "../../wit",
    world: "tairitsu",
});

#[cfg(target_family = "wasm")]
use exports::tairitsu::core::guest_api::Guest as GuestApi;

#[cfg(target_family = "wasm")]
struct GuestImpl;

#[cfg(target_family = "wasm")]
impl GuestApi for GuestImpl {
    fn init() -> Result<(), String> {
        // ✅ Guest 调用 Host API - 记录日志
        tairitsu::core::host_api::log("info", "Guest module initialized");
        
        // ✅ Guest 调用 Host API - 执行命令获取信息
        match tairitsu::core::host_api::execute("get_info", "{}") {
            Ok(response) => {
                tairitsu::core::host_api::log("info", &format!("Host info: {}", response));
                Ok(())
            }
            Err(e) => {
                tairitsu::core::host_api::log("error", &format!("Failed to get host info: {}", e));
                Err(e)
            }
        }
    }
    
    fn handle_command(command: String, payload: String) -> Result<String, String> {
        // ✅ Guest 调用 Host API - 记录接收到的命令
        tairitsu::core::host_api::log(
            "info",
            &format!("Received command: {} with payload: {}", command, payload),
        );
        
        match command.as_str() {
            "greet" => {
                Ok(format!("Hello from WASM guest! You said: {}", payload))
            }
            "compute" => {
                let result = payload.len() * 42;
                Ok(format!("Computed result: {}", result))
            }
            "call_host" => {
                // ✅ Guest 调用 Host API - 回调到 Host 执行命令
                match tairitsu::core::host_api::execute("echo", &payload) {
                    Ok(host_response) => Ok(format!("Host echoed: {}", host_response)),
                    Err(e) => Err(format!("Host call failed: {}", e)),
                }
            }
            _ => Err(format!("Unknown command: {}", command)),
        }
    }
}

#[cfg(target_family = "wasm")]
export!(GuestImpl);
```

**Guest 端功能**：
1. **导出 `guest-api` 接口**：实现 `init()` 和 `handle_command()` 供 Host 调用
2. **导入 `host-api` 接口**：调用 Host 的 `log()` 和 `execute()` 函数
3. **双向通信示例**：
   - 在 `init()` 中调用 Host 的 `execute("get_info")`
   - 在 `handle_command("call_host")` 中调用 Host 的 `execute("echo")`

## 3. Host 端实现（Native）

### 位置：`examples/hybrid/src/host.rs`

```rust
use tairitsu::{Container, Registry};

fn main() -> Result<()> {
    // 1. 加载 WASM 模块
    let wasm_binary = std::fs::read(&wasm_path).map(Bytes::from)?;
    
    // 2. 创建 Registry 并注册 Image
    let registry = Registry::new();
    registry.register_image("hybrid-example:latest", wasm_binary)?;
    
    // 3. 从 Image 创建 Container
    let image = registry.get_image("hybrid-example:latest").unwrap();
    let mut container = Container::new(&image)?;
    
    // 4. ✅ 设置 Host 端处理器 - 实现 host-api 接口
    container.on_execute(|command: String, payload: String| {
        println!("[Host] Received execute request: command='{}', payload='{}'", command, payload);
        match command.as_str() {
            "get_info" => {
                Ok(r#"{"name":"Tairitsu Host","version":"0.1.0","status":"running"}"#.to_string())
            }
            "echo" => Ok(payload),
            _ => Err(format!("Unknown host command: {}", command)),
        }
    });
    
    container.on_log(|level: String, message: String| {
        println!("[Guest Log][{}] {}", level.to_uppercase(), message);
    });
    
    // 5. ✅ Host 调用 Guest API - 初始化
    container.init()?;
    
    // 6. ✅ Host 调用 Guest API - 发送命令
    let commands = vec![
        ("greet", "Tairitsu Framework"),
        ("compute", "Hello World"),
        ("call_host", "This message goes to host and back"),
    ];
    
    for (cmd, payload) in commands {
        match container.handle_command(cmd, payload) {
            Ok(response) => {
                println!("[Host] Guest response: {}", response);
            }
            Err(e) => {
                eprintln!("[Host] Guest error: {}", e);
            }
        }
    }
    
    Ok(())
}
```

**Host 端功能**：
1. **加载和管理 WASM 模块**：使用 Registry/Image/Container 模式
2. **实现 `host-api` 接口**：通过 `on_execute()` 和 `on_log()` 注册处理器
3. **调用 `guest-api` 接口**：调用 `container.init()` 和 `container.handle_command()`

## 4. 运行时验证日志

### 构建命令：
```bash
# 构建 WASM Guest 模块
cargo build --target wasm32-wasip1 --release --package tairitsu-example-hybrid

# 运行 Native Host
cargo run --package tairitsu-example-hybrid --bin host --features host-binary --release
```

### 完整运行日志：

```
=== Tairitsu Hybrid Example - Host Side ===

Loading WASM module from: /home/runner/work/tairitsu/tairitsu/examples/hybrid/../../target/wasm32-wasip1/release/tairitsu_example_hybrid.wasm
WASM module loaded (55022 bytes)

Registering WASM module as image 'hybrid-example:latest'
Creating container from image...

=== Initializing Guest Module ===
[Guest Log][INFO] Guest module initialized                    ← Guest 调用 host-api::log()
[Host] Received execute request: command='get_info', payload='{}'  ← Guest 调用 host-api::execute()
[Guest Log][INFO] Host info: {"name":"Tairitsu Host","version":"0.1.0","status":"running"}  ← Guest 记录 Host 返回的数据

=== Sending Commands to Guest ===

[Host] Sending command: 'greet' with payload: 'Tairitsu Framework'  ← Host 调用 guest-api::handle_command()
[Guest Log][INFO] Received command: greet with payload: Tairitsu Framework  ← Guest 记录收到的命令
[Host] Guest response: Hello from WASM guest! You said: Tairitsu Framework  ← Host 收到 Guest 响应

[Host] Sending command: 'compute' with payload: 'Hello World'  ← Host 调用 guest-api::handle_command()
[Guest Log][INFO] Received command: compute with payload: Hello World  ← Guest 记录收到的命令
[Host] Guest response: Computed result: 462  ← Host 收到 Guest 响应

[Host] Sending command: 'call_host' with payload: 'This message goes to host and back'  ← Host 调用 guest-api::handle_command()
[Guest Log][INFO] Received command: call_host with payload: This message goes to host and back  ← Guest 记录收到的命令
[Host] Received execute request: command='echo', payload='This message goes to host and back'  ← Guest 调用 host-api::execute()
[Host] Guest response: Host echoed: This message goes to host and back  ← Host 收到 Guest 响应（包含 Host echo 的结果）

=== Example Complete ===

This example demonstrated:
1. Loading a WASM module into an Image (like docker pull/build)
2. Creating a Container from the Image (like docker run)
3. Bidirectional communication:
   - Host calling Guest (via handle_command)
   - Guest calling Host (via execute API)
4. Shared WIT interface definitions for type-safe communication
```

## 5. 双向通信流程分析

### 5.1 Host → Guest 调用

**调用链路**：
```
Host (host.rs)
  └─ container.init()
      └─ WIT binding
          └─ WASM Guest (lib.rs)
              └─ GuestImpl::init()
```

**日志证明**：
```
[Host] Sending command: 'greet' with payload: 'Tairitsu Framework'
[Guest Log][INFO] Received command: greet with payload: Tairitsu Framework
[Host] Guest response: Hello from WASM guest! You said: Tairitsu Framework
```

### 5.2 Guest → Host 调用

**调用链路**：
```
WASM Guest (lib.rs)
  └─ tairitsu::core::host_api::execute("get_info", "{}")
      └─ WIT binding
          └─ Host (host.rs)
              └─ on_execute handler
```

**日志证明**：
```
[Host] Received execute request: command='get_info', payload='{}'
[Guest Log][INFO] Host info: {"name":"Tairitsu Host","version":"0.1.0","status":"running"}
```

### 5.3 嵌套双向调用

**调用链路**：
```
Host → Guest (call_host command)
  └─ Guest → Host (execute "echo")
      └─ Host 返回结果
  └─ Guest 返回最终结果给 Host
```

**日志证明**：
```
[Host] Sending command: 'call_host' with payload: 'This message goes to host and back'
[Guest Log][INFO] Received command: call_host with payload: This message goes to host and back
[Host] Received execute request: command='echo', payload='This message goes to host and back'
[Host] Guest response: Host echoed: This message goes to host and back
```

## 6. 验证结论

✅ **WIT 接口定义正确**：定义了清晰的 `guest-api` 和 `host-api` 接口

✅ **Host → Guest 通信验证**：
- Host 成功调用 Guest 的 `init()` 函数
- Host 成功调用 Guest 的 `handle_command()` 函数
- 所有调用都通过 WIT Component Model 类型安全地传递

✅ **Guest → Host 通信验证**：
- Guest 成功调用 Host 的 `log()` 函数（多次）
- Guest 成功调用 Host 的 `execute()` 函数（get_info、echo）
- 所有调用都正确接收返回值

✅ **双向嵌套调用验证**：
- 在 Guest 处理 Host 请求时，Guest 可以回调 Host
- 数据在 Host ↔ Guest 之间正确流转

✅ **类型安全保证**：
- 所有参数和返回值都通过 WIT 定义的类型
- 编译时检查确保接口匹配

## 7. 框架特性总结

1. **Docker-like 架构**：Image（镜像）→ Container（容器）→ Registry（注册表）
2. **WIT Component Model**：使用 WebAssembly Component Model 的标准接口
3. **完全双向通信**：Host 和 Guest 可以相互调用，支持嵌套调用
4. **类型安全**：编译时类型检查，运行时类型转换
5. **易于使用**：简洁的 API，符合 Rust 习惯用法

## 8. 源码文件位置

- WIT 接口：`wit/world.wit`
- Guest 实现：`examples/hybrid/src/lib.rs`
- Host 实现：`examples/hybrid/src/host.rs`
- 框架核心：`packages/vm/src/`
  - `image.rs` - Image 管理
  - `container.rs` - Container 运行时
  - `registry.rs` - Registry 注册表
