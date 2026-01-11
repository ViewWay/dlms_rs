# SNRM/UA 握手实现设计方案

## 问题分析

根据 `dlms-docs/dlms/cosem连接过程.txt` 和 `长数据帧处理.txt` 文档：

### 连接建立流程
```
客户端 -> SNRM -> 服务器
客户端 <- UA <- 服务器
```

### UA帧参数（根据长数据帧处理.txt）
UA帧的信息域包含以下链路参数：
- Window_size: 通讯的双方一次发送数据帧的数目（默认值为1）
- Maximum_information_field_length: 链路数据帧中用户数据的最大长度（默认值为128）
- Transmit_maximum_information_field_length
- Receive_maximum_information_field_length
- Transmit_window_size
- Receive_window_size

## 设计方案

### 1. UA帧参数结构

UA帧的信息域格式需要定义。根据DLMS标准，UA帧的信息域通常包含：
- 格式标识符（Format Identifier）
- 组标识符（Group Identifier）
- 参数值

但根据文档，我们需要解析的参数是：
- Window_size (u8)
- Maximum_information_field_length (u16)

### 2. 实现步骤

#### 步骤1: 定义UA帧参数结构
```rust
pub struct UaFrameParameters {
    pub window_size: u8,
    pub max_information_field_length: u16,
    // 可选：其他参数
}
```

#### 步骤2: 实现UA帧参数编码/解码
- 编码：将参数编码为字节序列
- 解码：从UA帧信息域解析参数

#### 步骤3: 修改open()方法
1. 打开传输层
2. 发送SNRM帧
3. 等待UA响应（带超时）
4. 解析UA帧中的参数
5. 更新HdlcParameters

#### 步骤4: 错误处理
- SNRM发送失败
- UA响应超时
- UA帧格式错误
- 参数解析失败

### 3. UA帧信息域格式

根据DLMS标准（IEC 62056-47），UA帧的信息域格式通常是：
- 格式标识符：1字节（通常为0x81）
- 组标识符：1字节（通常为0x80）
- 参数值：可变长度

但根据文档描述，可能需要简化的格式。需要进一步确认标准。

### 4. 实现细节

#### SNRM帧发送
- 使用FrameType::SetNormalResponseMode
- 信息域可以为空（根据标准）

#### UA帧接收
- 等待FrameType::UnnumberedAcknowledge
- 解析信息域中的参数
- 验证参数有效性

#### 参数验证
- Window_size: 1-7（HDLC窗口大小限制）
- Maximum_information_field_length: > 0

### 5. 优化考虑

1. **超时处理**: 使用可配置的超时时间（默认5秒）
2. **重试机制**: 可选的SNRM重试（如果需要）
3. **参数协商**: 如果服务器返回的参数与客户端期望不符，可以选择拒绝或接受

### 6. 错误处理策略

- **传输层错误**: 返回Connection错误
- **SNRM发送失败**: 返回Connection错误
- **UA响应超时**: 返回Timeout错误
- **UA帧格式错误**: 返回FrameInvalid错误
- **参数无效**: 返回InvalidData错误

## 风险评估

- **中等风险**: 修改连接建立流程可能影响现有代码
- **缓解措施**: 
  - 添加详细的错误处理
  - 提供配置选项（超时时间等）
  - 添加单元测试和集成测试

## 测试计划

1. **单元测试**:
   - UA帧参数编码/解码
   - 参数验证

2. **集成测试**:
   - 完整的SNRM/UA握手流程
   - 超时处理
   - 错误情况处理

3. **兼容性测试**:
   - 与标准DLMS设备通信
   - 不同参数值测试
