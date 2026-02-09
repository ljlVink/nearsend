# 迁移到使用 LocalSend 官方 core

## 为什么使用 LocalSend 官方 core？

1. **100% 协议兼容性**：直接使用官方实现，确保完全兼容
2. **官方维护**：由 LocalSend 官方团队维护，跟随协议更新
3. **减少代码量**：不需要重新实现协议细节
4. **经过充分测试**：官方代码经过大量测试和实际使用
5. **功能完整**：包含完整的协议实现，包括 HTTP、WebRTC 等

## 迁移步骤

### 1. 添加依赖

已在 `Cargo.toml` 中添加（使用官方仓库的 core crate）：
```toml
localsend = { git = "https://github.com/localsend/localsend.git", features = ["http"] }
```

**为什么使用官方 core？**
- LocalSend 官方仓库有一个独立的 `core` crate
- `server` 目录也使用这个 core：`localsend = { path = "../core" }`
- 这是最权威的实现，保证协议兼容性
- 官方维护，持续更新

### 2. 移除自定义协议实现

以下文件可以移除或标记为废弃：
- `src/protocol.rs` - 协议定义（由 localsend core 提供）
- `src/discovery.rs` - 发现服务（由 localsend core 提供）
- `src/server.rs` - 服务器实现（由 localsend core 提供）
- `src/client.rs` - 客户端实现（由 localsend core 提供）

### 3. 使用 LocalSend core API

参考 `src/app_v2.rs` 中的新实现方式。

### 4. 保留 UI 部分

我们的 GPUI UI 实现保持不变，只需要：
- 使用 `localsend` crate 提供的类型替代自定义实现
- 使用 `localsend` 的 API 进行设备发现和文件传输

## LocalSend core API 使用示例

根据官方 core 的结构，主要模块包括：

```rust
use localsend::http::client::*;  // HTTP 客户端
use localsend::http::server::*;  // HTTP 服务器
use localsend::model::*;         // 数据模型（discovery, transfer）
use localsend::crypto::*;        // 加密相关（需要 crypto feature）
```

**模块结构：**
- `http/client/` - HTTP 客户端实现
- `http/server/` - HTTP 服务器实现
- `model/discovery.rs` - 设备发现相关模型
- `model/transfer.rs` - 文件传输相关模型
- `crypto/` - 加密功能（ed25519, RSA, SHA2）

**Features 说明：**
- `crypto`: 加密功能（ed25519, RSA, SHA2）
- `http`: HTTP 客户端和服务器功能（包含 crypto）
- `webrtc`: WebRTC 支持（可选）
- `full`: 所有功能

**参考官方实现：**
- Core crate: https://github.com/localsend/localsend/tree/main/core
- Server 示例（使用 core）: https://github.com/localsend/localsend/tree/main/server
- Core Cargo.toml: https://github.com/localsend/localsend/blob/main/core/Cargo.toml
- Core lib.rs: https://github.com/localsend/localsend/blob/main/core/src/lib.rs

## 下一步

1. 查看 `localsend-rs` 的完整 API 文档
2. 重构 `app.rs` 使用 `localsend-rs`
3. 测试与官方 LocalSend 客户端的兼容性
4. 移除旧的协议实现代码
