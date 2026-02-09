# NearSend 架构设计

## 项目结构

```
src/
├── lib.rs                 # 入口点
├── app.rs                 # 主应用入口
├── core/                  # 核心业务逻辑
│   ├── mod.rs
│   ├── discovery.rs       # 设备发现服务
│   ├── transfer.rs        # 文件传输服务
│   └── server.rs          # HTTP 服务器管理
├── ui/                    # UI 层
│   ├── mod.rs
│   ├── pages/             # 页面组件
│   │   ├── mod.rs
│   │   ├── home.rs        # 主页（Tab 容器）
│   │   ├── send.rs        # 发送页面
│   │   ├── receive.rs     # 接收页面
│   │   └── progress.rs    # 传输进度页面
│   ├── components/        # 可复用组件
│   │   ├── mod.rs
│   │   ├── device_list.rs # 设备列表
│   │   ├── file_list.rs   # 文件列表
│   │   └── transfer_item.rs # 传输项
│   └── theme.rs           # 主题和样式
├── state/                 # 状态管理
│   ├── mod.rs
│   ├── app_state.rs       # 应用全局状态
│   ├── device_state.rs    # 设备状态
│   └── transfer_state.rs  # 传输状态
└── util/                  # 工具函数
    ├── mod.rs
    └── file_picker.rs     # 文件选择器（TODO）
```

## 核心功能模块

### 1. 设备发现 (core/discovery.rs)
- 使用 `localsend::model::discovery` 进行设备发现
- 管理设备列表和状态
- 处理设备上线/下线

### 2. 文件传输 (core/transfer.rs)
- 使用 `localsend::http::client` 发送文件
- 使用 `localsend::http::server` 接收文件
- 管理传输进度和状态

### 3. HTTP 服务器 (core/server.rs)
- 使用 `localsend::http::server` 启动服务器
- 处理接收请求
- 管理服务器生命周期

## UI 设计规范

### 移动端样式
- 使用卡片式设计
- 大按钮，易于点击
- 清晰的视觉层次
- 响应式布局

### 主要页面

1. **Home Page (主页)**
   - Tab 切换：发送 / 接收
   - 底部导航栏

2. **Send Page (发送页)**
   - 设备列表（卡片式）
   - 文件选择按钮
   - 已选文件列表
   - 发送按钮

3. **Receive Page (接收页)**
   - 接收状态显示
   - 接收历史
   - 设置选项

4. **Progress Page (进度页)**
   - 传输列表
   - 进度条
   - 取消/重试按钮

## 状态管理

使用 GPUI Entity 模式管理状态：
- `AppState`: 全局应用状态
- `DeviceState`: 设备列表和发现状态
- `TransferState`: 传输任务状态

## 协议兼容性

- 完全使用 `localsend` core crate
- 确保与官方 LocalSend 客户端兼容
- 遵循 LocalSend Protocol v2
