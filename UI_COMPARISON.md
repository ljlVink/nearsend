# UI Comparison: nearsend vs localsend

## ReceiveTab Differences

1. **Logo**: 
   - localsend: Uses `LocalSendLogo` component with rotation animation
   - nearsend: Uses placeholder "NS" text
   - **Fix**: Need to create Logo component and add rotation animation

2. **IP Display**:
   - localsend: Shows multiple IPs: `vm.localIps.map((ip) => '#${ip.visualId}').toSet().join(' ')`
   - nearsend: Shows single IP
   - **Fix**: Need to support multiple IPs

3. **Info Box Layout**:
   - localsend: Uses `Table` layout with proper spacing
   - nearsend: Uses simple `h_flex`
   - **Fix**: Change to Table-like layout

4. **Corner Buttons Animation**:
   - localsend: Uses `AnimatedOpacity` for history button
   - nearsend: Simple conditional rendering
   - **Fix**: Add animation support

5. **History Button Visibility**:
   - localsend: Uses `showHistoryButton` flag
   - nearsend: Uses `!show_advanced` condition
   - **Fix**: Add proper `showHistoryButton` logic

## SendTab Differences

1. **File Selection Buttons**:
   - localsend: Uses `BigButton` component (icon + label, specific size)
   - nearsend: Uses simple `Button` with emoji
   - **Fix**: Create `BigButton` component

2. **File Thumbnails**:
   - localsend: Uses `SmartFileThumbnail` with actual file previews
   - nearsend: Uses placeholder "File N" text
   - **Fix**: Implement file thumbnail component

3. **Device List**:
   - localsend: Uses `DeviceListTile` with badges, favorites, progress
   - nearsend: Uses `DeviceCard` (simpler)
   - **Fix**: Align `DeviceCard` with `DeviceListTile` design

4. **Scan Button**:
   - localsend: Has rotation animation and IP selection popup
   - nearsend: Simple button
   - **Fix**: Add animation and popup menu

5. **Send Mode Button**:
   - localsend: Has popup menu with Single/Multiple/Link modes
   - nearsend: Not implemented
   - **Fix**: Implement send mode selection

6. **Placeholder**:
   - localsend: Uses `DevicePlaceholderListTile` when no devices
   - nearsend: Simple text message
   - **Fix**: Create placeholder component

7. **Help Text**:
   - localsend: Uses `OpacitySlideshow` animation
   - nearsend: Static text
   - **Fix**: Add slideshow animation

8. **Troubleshoot Button**:
   - localsend: Has troubleshoot button
   - nearsend: Missing
   - **Fix**: Add troubleshoot button

## SettingsTab Differences

1. **Layout Structure**:
   - localsend: Uses `_SettingsSection` and `_SettingsEntry` components
   - nearsend: Uses simple cards
   - **Fix**: Create proper section/entry components

2. **Switch Component**:
   - localsend: Uses `Switch` widget
   - nearsend: Uses `Button` to simulate switch
   - **Fix**: Use proper Switch component

3. **Missing Settings**:
   - localsend: Has brightness, color, language, destination, saveToGallery, etc.
   - nearsend: Missing many settings
   - **Fix**: Add all missing settings

4. **Advanced Settings**:
   - localsend: Has many advanced options (device type, model, port, network, etc.)
   - nearsend: Only shows placeholder text
   - **Fix**: Implement all advanced settings

5. **About Section**:
   - localsend: Has logo, version, copyright, changelog button
   - nearsend: Simple text
   - **Fix**: Add proper about section

## Static Resources

1. **Logo Images**: Copied ✓
2. **Fonts**: Need to check if fonts are needed
3. **Other Assets**: Need to check for other assets
