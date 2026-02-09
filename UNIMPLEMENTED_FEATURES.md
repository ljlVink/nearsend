# 未实现功能清单

## UI 层面未实现的功能

### ReceiveTab (接收页面)
- ❌ **Logo 组件**: 当前使用占位符 "NS" 文字，需要：
  - 集成实际的 logo 图片 (`assets/img/logo-512.png`)
  - 实现旋转动画（当服务器运行时）
- ❌ **历史记录页面**: History 按钮点击后需要导航到历史记录页面
- ⚠️ **服务器状态**: 当前使用硬编码值，需要从实际的服务器状态获取

### SendTab (发送页面)
- ❌ **文件选择器**: 
  - Photos 按钮：需要实现照片选择器
  - Videos 按钮：需要实现视频选择器  
  - Files 按钮：需要实现文件选择器
- ❌ **文件缩略图**: 当前显示 "File N"，需要实现实际的文件预览缩略图
- ❌ **文件大小计算**: 当前显示 "0 B"，需要计算选中文件的总大小
- ❌ **设备扫描**: Scan 按钮需要：
  - 实现实际的设备扫描功能
  - 添加旋转动画
  - 实现 IP 选择弹出菜单（当有多个网络接口时）
- ❌ **手动地址**: Manual 按钮需要显示手动输入地址的对话框
- ❌ **收藏夹**: Favorites 按钮需要显示收藏设备对话框
- ❌ **发送模式**: Send Mode 按钮需要实现弹出菜单（Single/Multiple/Link 模式）
- ❌ **发送到设备**: DeviceCard 的 on_select 需要实现实际的发送功能
- ❌ **故障排除页面**: Troubleshoot 按钮需要导航到故障排除页面
- ❌ **OpacitySlideshow**: 帮助文本需要实现动画轮播效果
- ❌ **编辑文件**: Edit 按钮需要导航到文件选择页面
- ❌ **添加文件**: Add 按钮需要添加更多文件

### SettingsTab (设置页面)
- ❌ **Switch 组件**: 当前使用 Button 模拟开关，需要使用真正的 Switch 组件
- ❌ **General 设置**:
  - Brightness (主题模式选择器)
  - Color (颜色模式选择器)
  - Language (语言选择器)
- ❌ **Receive 设置**:
  - Destination (下载目录选择器)
  - Save to Gallery (保存到相册开关)
- ❌ **Network 设置**:
  - Server 控制按钮 (Start/Restart/Stop)
  - Alias 输入框（可编辑，带随机生成和系统名称按钮）
  - Port 输入框（高级设置）
  - Device Type 选择器（高级设置）
  - Device Model 输入框（高级设置）
  - Network 过滤设置（高级设置）
  - Discovery Timeout 输入框（高级设置）
  - Encryption/HTTPS 开关（高级设置）
  - Multicast Group 输入框（高级设置）
  - Port 警告消息
  - Multicast Group 警告消息
- ❌ **Send 设置** (高级设置):
  - Share via Link Auto Accept 开关
- ❌ **Other 设置**:
  - About 页面按钮
  - Support/Donate 按钮
  - Privacy Policy 按钮
  - Terms of Use 按钮 (iOS/macOS)
- ❌ **About 部分**:
  - Logo with text
  - Copyright 文本
  - Changelog 按钮

### 通用组件缺失
- ❌ **OpacitySlideshow**: 动画轮播组件
- ❌ **Switch**: 真正的开关组件（gpui-component 可能有，需要检查）
- ❌ **Dropdown/Select**: 下拉选择组件
- ❌ **Text Input**: 文本输入组件
- ❌ **Dialog/Modal**: 对话框组件（用于 PIN、手动地址等）
- ❌ **File Picker**: 文件选择器集成（OpenHarmony API）
- ❌ **Directory Picker**: 目录选择器集成（OpenHarmony API）

## 功能层面未实现的功能

### 核心功能
- ❌ **设备发现服务**: `core/discovery.rs` 中的发现服务初始化
- ❌ **文件发送**: `core/transfer.rs` 中的文件发送实现
- ❌ **文件下载**: `server.rs` 中的文件下载实现
- ❌ **TLS 证书生成**: 使用 localsend::crypto 生成 TLS 证书

### 状态管理
- ❌ **服务器状态同步**: ReceiveTab 需要从实际的服务器状态获取信息
- ❌ **设备列表更新**: SendTab 需要从实际的设备发现服务获取设备列表
- ❌ **文件选择状态**: 需要实现文件选择后的状态管理和大小计算

## 代码中的 TODO 标记

根据代码扫描，发现以下 TODO：

### app.rs
- Line 64: Start discovery and server services
- Line 253: Navigate to history page
- Line 412, 420, 428: Implement file picker
- Line 548: Trigger actual device scan
- Line 591: Implement send to device
- Line 608: Navigate to troubleshoot page
- Line 785: Show PIN dialog if enabling

### ui/components/device_card.rs
- Line 84: Map device_type to actual icon

### ui/pages/send.rs
- Line 69: Implement photo picker
- Line 80: Implement video picker
- Line 91: Implement file picker
- Line 145: Calculate total size
- Line 177: Navigate to file selection page
- Line 186: Add more files
- Line 221: Trigger device scan
- Line 231: Show manual address dialog
- Line 241: Show favorites dialog
- Line 251: Show send mode menu
- Line 279: Render actual devices from device_state

### ui/pages/receive.rs
- Line 41: Get from actual server state
- Line 43: Get from actual server state
- Line 81: Add actual logo image
- Line 180: Navigate to history page

### ui/pages/progress.rs
- Line 43: Fetch active transfers from transfer_state

### ui/pages/home.rs
- Line 75, 79: Render SendPage/ReceivePage properly

### server.rs
- Line 100: Implement file download

### core/transfer.rs
- Line 18: Generate TLS certificate
- Line 32: Implement file sending

### core/server.rs
- Line 36: Generate TLS certificate

### core/discovery.rs
- Line 7, 21: Implement discovery mechanism

## 优先级建议

### 高优先级（核心功能）
1. 设备发现服务实现
2. 文件发送功能实现
3. 文件下载功能实现
4. 服务器状态同步

### 中优先级（重要 UI）
1. Logo 组件和旋转动画
2. 文件选择器集成
3. Switch 组件
4. SettingsTab 的完整设置项

### 低优先级（增强功能）
1. OpacitySlideshow 动画
2. 弹出菜单实现
3. 对话框组件
4. 文件缩略图预览
