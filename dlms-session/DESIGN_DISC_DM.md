# DISC/DM/UA 断开流程设计方案

## 问题分析

根据 `dlms-docs/dlms/cosem连接过程.txt` 和 `长数据帧处理.txt` 文档：

### 断开流程
```
客户端 -> DISC -> 服务器
客户端 <- DM/UA <- 服务器
```

### 响应说明（长数据帧处理.txt）
- **UA**: 表示接收到DISC帧后断开链接
- **DM**: 表示在接收到DISC帧之前就已经处于链路断开状态

## 设计方案

### 1. 断开流程步骤

1. 检查连接状态（如果已关闭，直接返回）
2. 发送DISC帧
3. 等待DM或UA响应（带超时）
4. 关闭传输层
5. 更新连接状态

### 2. 实现细节

#### DISC帧发送
- 使用FrameType::Disconnect
- 信息域为空（根据HDLC标准）

#### 响应处理
- 等待FrameType::DisconnectMode或FrameType::UnnumberedAcknowledge
- 超时处理：如果超时，仍然关闭连接（可能服务器已断开）
- 错误处理：如果接收失败，仍然尝试关闭传输层

#### 状态管理
- 设置`closed = true`
- 关闭传输层

### 3. 错误处理策略

- **DISC发送失败**: 仍然尝试关闭传输层
- **响应超时**: 仍然关闭连接（服务器可能已断开）
- **响应格式错误**: 记录错误但继续关闭
- **传输层关闭失败**: 返回错误

### 4. 优化考虑

1. **超时处理**: 默认3秒超时（比连接建立短，因为断开是单向操作）
2. **容错性**: 即使响应失败，也尝试关闭传输层
3. **状态一致性**: 确保连接状态和传输层状态一致

## 实现方案

### close()方法修改

```rust
pub async fn close(&mut self) -> DlmsResult<()> {
    if self.closed {
        return Ok(()); // Already closed
    }

    // Send DISC frame
    let address_pair = HdlcAddressPair::new(self.local_address, self.remote_address);
    let disc_frame = HdlcFrame::new(address_pair, FrameType::Disconnect, None);
    let _ = self.send_frame(disc_frame).await; // Ignore errors, try to close anyway

    // Wait for DM or UA response with timeout
    let timeout = Duration::from_secs(3);
    if let Ok(frames) = self.receive_frames(Some(timeout)).await {
        // Check for DM or UA frame
        let _ = frames.iter().find(|f| {
            matches!(
                f.frame_type(),
                FrameType::DisconnectMode | FrameType::UnnumberedAcknowledge
            )
        });
        // Note: We don't fail if response is not received, as server may have already disconnected
    }

    // Close transport layer
    self.transport.close().await?;
    self.closed = true;
    Ok(())
}
```

## 测试计划

1. **单元测试**:
   - 正常断开流程（收到DM/UA）
   - 超时情况（未收到响应）
   - 已关闭连接的情况

2. **集成测试**:
   - 完整的断开流程
   - 错误情况处理

## 风险评估

- **低风险**: 断开流程是单向操作，即使失败也不会影响太大
- **缓解措施**: 
  - 容错处理：即使响应失败也关闭传输层
  - 超时处理：避免无限等待
