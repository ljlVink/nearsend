# OpenHarmony 编译检查清单

## 当前状态

### ✅ 已修复的问题
1. ✅ 修复了 `discovery.rs` 中的字段错误（移除了不存在的 `multicast_discovery` 字段）
2. ✅ 修复了 `server.rs` 中的字段错误（移除了不存在的 `server` 字段）
3. ✅ 将 `app_new.rs` 中的 `tokio::spawn` 替换为 `cx.spawn`（GPUI 异步上下文）

### ⚠️ 潜在问题

#### 1. `localsend` core 的 tokio 依赖
- `localsend::http::server::start_with_port` 内部使用 `tokio::spawn`
- 这可能在 OpenHarmony 上需要特殊处理
- **状态**: 需要测试验证

#### 2. `server.rs` 中的 `tokio::spawn`
- `ServerManager::start()` 方法中仍使用 `tokio::spawn`
- 这可能需要改为使用 GPUI 的异步上下文
- **状态**: 需要根据实际编译错误调整

#### 3. `tokio::sync` 的使用
- 多个模块使用 `tokio::sync::RwLock` 和 `tokio::sync::oneshot`
- 这些应该可以在 OpenHarmony 上工作，如果 tokio runtime 可用
- **状态**: 需要测试验证

## 编译测试

### 测试命令
```bash
RUSTFLAGS=$(printf '--cfg\x1fgles') ohrs build --arch aarch
```

### 预期可能的问题

1. **tokio runtime 问题**
   - 如果 `localsend` core 需要 tokio runtime，可能需要确保运行时正确初始化
   - 解决方案：确保 tokio runtime 在应用启动时初始化

2. **网络功能问题**
   - OpenHarmony 可能对网络功能有特殊要求
   - 解决方案：检查权限配置和网络 API 兼容性

3. **依赖编译问题**
   - `localsend` core 的某些依赖可能在 OpenHarmony 上不可用
   - 解决方案：检查并替换不兼容的依赖

## 修复建议

### 如果遇到 tokio::spawn 问题

1. **选项 1**: 确保 tokio runtime 在应用启动时初始化
   ```rust
   // 在应用入口初始化 tokio runtime
   #[tokio::main]
   async fn main() {
       // ...
   }
   ```

2. **选项 2**: 使用 GPUI 的异步上下文包装
   ```rust
   cx.spawn(async move |cx: &mut AsyncApp| {
       // 在这里调用需要 tokio 的代码
   });
   ```

3. **选项 3**: 如果 localsend core 内部使用 tokio，可能需要：
   - 确保 tokio runtime 可用
   - 或者修改 localsend core 以支持 GPUI 的异步上下文（不推荐）

### 如果遇到网络功能问题

1. 检查 OpenHarmony 的网络权限配置
2. 确保 UDP 多播功能可用
3. 检查防火墙和网络配置

## 下一步

1. 运行编译测试：`RUSTFLAGS=$(printf '--cfg\x1fgles') ohrs build --arch aarch`
2. 根据编译错误进行修复
3. 如果 localsend core 有兼容性问题，考虑：
   - 使用条件编译
   - 创建适配层
   - 或者暂时禁用某些功能

## TODO

- [ ] 测试编译
- [ ] 修复编译错误
- [ ] 验证 tokio runtime 可用性
- [ ] 测试网络功能
- [ ] 验证 localsend core 兼容性
