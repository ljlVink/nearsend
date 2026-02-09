# NearSend 实现计划

## 当前状态

### ✅ 已完成
1. 项目结构设计
2. 核心模块框架（core/discovery, core/server, core/transfer）
3. 状态管理模块（state/app_state, state/device_state, state/transfer_state）
4. 使用官方 localsend core crate

### 🔄 进行中
1. 主应用重构
2. UI 实现

### 📋 TODO

#### 核心功能
- [ ] 实现设备发现（使用 localsend core）
- [ ] 实现 HTTP 服务器启动和管理
- [ ] 实现文件发送功能
- [ ] 实现文件接收功能
- [ ] TLS 证书生成（使用 localsend::crypto）

#### UI 实现
- [ ] 主页面（Tab 切换：发送/接收）
- [ ] 发送页面（设备列表、文件选择）
- [ ] 接收页面（接收状态、历史）
- [ ] 传输进度页面
- [ ] 设备列表组件
- [ ] 文件列表组件
- [ ] 传输项组件

#### 移动端样式
- [ ] 卡片式设计
- [ ] 大按钮样式
- [ ] 响应式布局
- [ ] 移动端导航

#### 编译和测试
- [ ] 确保 `ohrs build --arch aarch` 编译通过
- [ ] 测试与官方 LocalSend 客户端兼容性
- [ ] 文件选择器集成（需要平台支持）

## 实现步骤

### Phase 1: 核心功能（当前）
1. ✅ 项目结构设计
2. ✅ 核心模块框架
3. 🔄 主应用重构
4. ⏳ 设备发现实现
5. ⏳ 服务器实现
6. ⏳ 传输功能实现

### Phase 2: UI 实现
1. ⏳ 主页面布局
2. ⏳ 发送页面
3. ⏳ 接收页面
4. ⏳ 传输进度页面

### Phase 3: 样式和优化
1. ⏳ 移动端样式
2. ⏳ 响应式布局
3. ⏳ 用户体验优化

### Phase 4: 测试和发布
1. ⏳ 编译测试
2. ⏳ 兼容性测试
3. ⏳ 性能优化

## 技术债务

1. **文件选择器**: 需要平台特定的文件选择器实现（TODO）
2. **TLS 证书**: 需要实现证书生成逻辑
3. **设备发现**: 需要实现多播发现机制
4. **错误处理**: 需要完善错误处理和用户提示

## 参考资源

- LocalSend Core: https://github.com/localsend/localsend/tree/main/core
- LocalSend Server: https://github.com/localsend/localsend/tree/main/server
- LocalSend App: https://github.com/localsend/localsend/tree/main/app
