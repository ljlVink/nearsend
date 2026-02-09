# UI Alignment Progress: nearsend vs localsend

## ✅ Completed Fixes

### ReceiveTab
- ✅ Multiple IP display support (matching localsend's `vm.localIps.map((ip) => '#${ip.visualId}').toSet().join(' ')`)
- ✅ Info Box layout changed to Table-like format matching localsend
- ✅ Added `show_history_button` logic
- ✅ Improved Info Box to support multiple IPs display

### SendTab
- ✅ Created `BigButton` component matching localsend's BigButton design
- ✅ Added file size formatting function (`format_file_size`)
- ✅ Improved `DeviceCard` to match `DeviceListTile` design:
  - Added device icon support
  - Added badges (LAN • HTTP, WebRTC, device model)
  - Added progress bar support
  - Added favorite button support
  - Added info text support
- ✅ Created `DevicePlaceholder` component matching localsend's DevicePlaceholderListTile
- ✅ Added Troubleshoot button
- ✅ File size display in file selection card

### Components Created
- ✅ `DeviceBadge` - Badge component matching localsend
- ✅ `ProgressBar` - Progress bar component matching localsend
- ✅ `DevicePlaceholder` - Placeholder component for empty device list
- ✅ `BigButton` - Big button component for file selection
- ✅ `format_file_size` - Utility function for file size formatting

### Static Resources
- ✅ Copied logo images from localsend to `nearsend/assets/img/`

## 🔄 In Progress / Partially Complete

### SendTab
- ⚠️ File thumbnails: Created structure but need actual file preview implementation
- ⚠️ Scan button: Basic button exists but missing rotation animation and IP selection popup
- ⚠️ Send mode button: Button exists but missing popup menu with Single/Multiple/Link modes
- ⚠️ OpacitySlideshow: Help text exists but missing animation

### SettingsTab
- ⚠️ Switch component: Currently using Button to simulate switch, need proper Switch component
- ⚠️ Settings sections: Basic structure exists but missing many settings items

## ❌ Still Missing

### ReceiveTab
- ❌ Logo component with rotation animation (currently using placeholder "NS" text)
- ❌ Rotation animation support for logo when server is running

### SendTab
- ❌ File thumbnail component with actual file previews
- ❌ Scan button rotation animation and IP selection popup menu
- ❌ Send mode popup menu (Single/Multiple/Link modes)
- ❌ OpacitySlideshow animation component for help text

### SettingsTab
- ❌ Brightness setting (Theme mode selector)
- ❌ Color setting (Color mode selector)
- ❌ Language setting
- ❌ Destination setting (download directory picker)
- ❌ Save to Gallery setting
- ❌ Server control buttons (Start/Restart/Stop)
- ❌ Alias input field with random generation and system name buttons
- ❌ Device Type setting (advanced)
- ❌ Device Model setting (advanced)
- ❌ Port input field (advanced)
- ❌ Network filtering setting (advanced)
- ❌ Discovery Timeout setting (advanced)
- ❌ Encryption/HTTPS setting (advanced)
- ❌ Multicast Group setting (advanced)
- ❌ Port warning message
- ❌ Multicast group warning message
- ❌ Share via Link Auto Accept setting (advanced)
- ❌ About page button
- ❌ Support/Donate button
- ❌ Privacy Policy button
- ❌ Terms of Use button (iOS/macOS)
- ❌ Logo with text in About section
- ❌ Copyright text
- ❌ Changelog button
- ❌ Proper Switch component (currently using Button simulation)

### Components Needed
- ❌ OpacitySlideshow component
- ❌ Switch component (proper toggle)
- ❌ Dropdown/Select component
- ❌ Text input component
- ❌ File picker integration
- ❌ Directory picker integration

## Notes

1. **Static Resources**: Logo images have been copied. Need to integrate them into the UI.

2. **Animation Support**: GPUI may have different animation APIs than Flutter. Need to check gpui-component for animation support.

3. **File Picker**: Need to integrate with OpenHarmony file picker APIs.

4. **Settings Structure**: The settings page needs significant expansion to match localsend's comprehensive settings.

5. **Component Library**: Many components exist in gpui-component but may need customization to match localsend's exact design.

## Next Steps

1. Create OpacitySlideshow component
2. Create proper Switch component or use gpui-component's switch if available
3. Add all missing settings items to SettingsTab
4. Implement file thumbnail component
5. Add rotation animations
6. Implement popup menus for scan button and send mode button
7. Integrate logo images into UI
8. Add all advanced settings options
