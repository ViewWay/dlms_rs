# DLMS/COSEM 项目待办事项清单

本文档记录了DLMS/COSEM Rust实现项目的所有待办事项，按模块和优先级分类。

**最后更新**: 2026-01-15 (Session 3: 高优先级需求验证完成 - Conformance BER/COSEM-OPEN/RELEASE/AARQ-AARE封装/加密PDU/SN寻址PDU/ConfirmedServiceError)

---

## 🐛 Bug修复跟踪

**Bug报告位置**: `dlms-docs/bugreview/BUG_REPORT.md`

### 🔴 严重Bug (编译错误) - ✅ 已全部修复

| Bug | 文件 | 问题 | 状态 |
|-----|------|------|------|
| #1 | `dlms-core/src/obis_code.rs:124` | Display格式字符串缺少点号分隔符 | ✅ 已修复 |
| #2 | `dlms-session/src/hdlc/decoder.rs:106` | `read_exact`函数定义在impl块外部 | ✅ 已修复 |

### 🟠 中等Bug (功能问题) - ✅ 已全部修复

| Bug | 文件 | 问题 | 状态 |
|-----|------|------|------|
| #3 | `dlms-session/src/hdlc/window.rs:221` | Wrap-around时旧周期帧未被确认 | ✅ 已修复 |
| #4 | `dlms-transport/src/tcp.rs:72` | TCP timeout未应用于connect | ✅ 已修复 |
| #5 | `dlms-core/src/datatypes/cosem_date.rs:266` | 测试用例逻辑不一致 | ✅ 已修复 |

### 🟡 轻微Bug (代码质量) - ✅ 已全部修复

| Bug | 文件 | 问题 | 状态 |
|-----|------|------|------|
| #6 | `dlms-session/src/hdlc/connection.rs` | deprecated `closed`字段未清理 | ✅ 已修复 |
| #7 | `dlms-session/src/hdlc/connection.rs` | 不必要的`mut`绑定 | ✅ 已修复 |
| #8 | `dlms-session/src/hdlc/connection.rs` | 冗余状态检查 | ✅ 已修复 |

### Bug修复完成 (2026-01-15)

所有8个Bug已全部修复：
- 2个严重Bug (编译错误)
- 3个中等Bug (功能问题)
- 3个轻微Bug (代码质量)

---

## 📊 总体进度

- **已完成**: 核心协议栈（传输层、会话层、安全层、应用层基础功能）
- **进行中**: 服务器实现、COSEM接口类、COSEM-OPEN/RELEASE服务
- **待实现**: 加密PDU、SN寻址PDU、更多接口类、高级功能

---

## 🔥 来自dlms-docs的需求汇总

根据 `dlms-docs/requirements/` 目录下的需求文档整理：

### 📋 需求文档符合度总览

| 需求类别 | 完成度 | 详细文档 |
|---------|--------|----------|
| 术语实现 | 75% | [TERMS_IMPLEMENTATION.md](dlms-docs/TERMS_IMPLEMENTATION.md) |
| COSEM-OPEN服务 | 95% | [COSEM_OPEN_ANALYSIS.md](dlms-docs/COSEM_OPEN_ANALYSIS.md) |
| COSEM-RELEASE服务 | 95% | [REQUIREMENTS_SUMMARY.md](dlms-docs/requirements/COSEM_RELEASE_REQUIREMENTS.md) |
| COSEM PDU ASN.1 | 90% | [COSEM_PDU_ASN1_REQUIREMENTS.md](dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md) |
| HDLC符合性 | 95% | [IMPLEMENTATION_COMPLIANCE_CHECK.md](dlms-docs/IMPLEMENTATION_COMPLIANCE_CHECK.md) |
| C++实现对比 | 70% | [IMPLEMENTATION_COMPARISON_REPORT.md](dlms-docs/IMPLEMENTATION_COMPARISON_REPORT.md) |

### 🔴 最高优先级需求（影响协议符合度）

#### 1. Conformance编码方式 ✅ 已完成
- **实现**: Conformance已支持BER编码（`encode_ber()`/`decode_ber()`）
- **说明**: InitiateRequest/Response内部使用BER编码Conformance字段
- **文档**: `dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md`
- **位置**: `dlms-application/src/pdu.rs` (Conformance编码/解码)
- **状态**: ✅ 已实现

#### 2. COSEM-OPEN服务原语 ✅ 已完成
- **实现**: Association模块提供完整的COSEM-OPEN服务
  - `build_aarq()` - 构建AARQ请求（COSEM-OPEN.request）
  - `process_aare()` - 处理AARE响应（COSEM-OPEN.confirm）
  - `open()` - 高级OPEN服务原语
  - `transition_to()` - 状态转换管理
- **文档**: `dlms-docs/COSEM_OPEN_ANALYSIS.md`
- **位置**: `dlms-application/src/association/mod.rs`
- **状态**: ✅ 已实现

#### 3. COSEM-RELEASE服务原语 ✅ 已完成
- **实现**: Association模块提供完整的COSEM-RELEASE服务
  - `build_rlrq()` - 构建RLRQ请求（COSEM-RELEASE.request）
  - `process_rlre()` - 处理RLRE响应（COSEM-RELEASE.confirm）
  - `release()` - 高级RELEASE服务原语
  - `abort()` - ABORT指示处理
- **文档**: `dlms-docs/requirements/COSEM_RELEASE_REQUIREMENTS.md`
- **位置**: `dlms-application/src/association/mod.rs`
- **状态**: ✅ 已实现

#### 4. AARQ/AARE封装 ✅ 已完成
- **实现**: InitiateRequest正确封装在AARQ的user_Information域
  - AARQ编码: `build_aarq()` 将InitiateRequest编码后插入user_information
  - AARE解码: `process_aare()` 从user_information提取InitiateResponse
  - 支持完整的应用层关联建立流程
- **文档**: `dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md`
- **位置**: `dlms-application/src/association/mod.rs`
- **状态**: ✅ 已实现

### 🟡 高优先级需求（功能完整性）

#### 5. 加密PDU支持 ✅ 已完成
- **实现**: 完整的全局加密(glo-*)和专用加密(ded-*)PDU支持
  - SecurityControl: 安全控制字节（1字节，包含密钥类型和PDU类型）
  - KeyType: Global/Dedicated密钥类型
  - EncryptedPduType: 17种PDU类型（0-16）
  - GlobalEncryptedPdu: 全局加密PDU结构
  - DedicatedEncryptedPdu: 专用加密PDU结构
  - EncryptedPdu: 统一的加密PDU枚举
  - 完整的encode/decode支持，包含system_title、frame_counter、encrypted_data、authentication_tag
- **文档**: `dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md`
- **位置**: `dlms-application/src/encrypted.rs`
- **状态**: ✅ 已实现

#### 6. SN寻址PDU ✅ 已完成
- **实现**: 完整的Short Name寻址PDU支持
  - SnPduTag: SN PDU标签枚举（6种PDU类型）
  - ShortName: 16位短名称地址类型
  - ReadRequest/ReadResponse: SN读取请求/响应
  - WriteRequest/WriteResponse: SN写入请求/响应
  - UnconfirmedWriteRequest: SN非确认写入请求
  - InformationReportRequest: SN信息报告请求
  - SnPdu: 统一的SN PDU枚举，支持自动解码
  - 完整的A-XDR编码/解码支持
- **文档**: `dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md`
- **位置**: `dlms-application/src/sn_pdu.rs`
- **状态**: ✅ 已实现

#### 7. ConfirmedServiceError ✅ 已完成
- **实现**: 完整的服务错误处理PDU
  - ServiceError: 9种错误类型（application-reference, hardware-resource, vde-state-error, service, definition, access, initiate, load-data-set, task）
  - ConfirmedServiceError: 19种服务错误类型（initiateError, getStatus, getNameList, read, write等）
  - 完整的A-XDR编码/解码支持
  - 详细的错误描述和诊断信息
- **文档**: `dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md`
- **位置**: `dlms-application/src/pdu.rs`
- **状态**: ✅ 已实现

### 🟢 中优先级需求（增强功能）

#### 8. LLC Header处理 ✅ 已完成
- **实现**: 完整的LLC header处理
  - LLC_REQUEST [0xE6, 0xE6, 0x00] 用于客户端请求
  - LLC_RESPONSE [0xE6, 0xE7, 0x00] 用于服务端响应
  - use_llc_header标志控制是否使用LLC header
  - send_information方法自动添加LLC header
  - receive_segmented方法自动移除LLC header
- **文档**: `dlms-docs/IMPLEMENTATION_COMPARISON_REPORT.md`
- **位置**: `dlms-session/src/hdlc/connection.rs`
- **状态**: ✅ 已实现

#### 9. ISO-ACSE高级功能 ✅ 已完成
- **实现**: 完整的AssociateSourceDiagnostic CHOICE支持
  - AcseServiceUser (tag 0) - 应用层错误诊断
  - AcseServiceProvider (tag 1) - 协议层错误诊断
  - 完整的BER编码/解码支持
  - AcseServiceUserDiagnostic - 16种标准诊断码常量
  - AcseServiceProviderDiagnostic - 3种标准诊断码常量
- **文档**: `dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md`
- **位置**: `dlms-asn1/src/iso_acse/types.rs`
- **状态**: ✅ 已实现

#### 10. Data-Notification PDU ✅ 已完成
- **实现**: 完整的DataNotification PDU支持
  - DataNotification结构: variable_name_specification (可选) + data_value
  - VariableNameSpecification CHOICE类型
    - CosemAttribute: COSEM属性引用 (LN寻址)
    - Structure: 复杂变量名结构 (保留用于扩展)
  - 完整的A-XDR编码/解码支持
  - 便捷构造方法: with_value(), with_attribute()
- **文档**: `dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md`
- **位置**: `dlms-application/src/pdu.rs`
- **状态**: ✅ 已实现
- **注**: InformationReportRequest已在SN PDU模块中实现

### 📌 低优先级需求（待确认）

#### 11. SHA-1算法 ❓
- **问题**: 当前使用SHA-256，需要确认DLMS/COSEM是否要求SHA-1
- **需求**: 如果协议要求，需要实现SHA-1
- **文档**: `dlms-docs/requirements/SHA1_ALGORITHM_REQUIREMENTS.md`
- **状态**: ❓ 待确认

### ✅ 已完成的重要功能

以下功能已根据文档要求完成实现：

1. **✅ HCS (Header Check Sequence)** - `dlms-session/src/hdlc/frame.rs`
2. **✅ HDLC连接建立流程 (SNRM/UA)** - `dlms-session/src/hdlc/connection.rs`
3. **✅ UA帧参数解析** - `dlms-session/src/hdlc/connection.rs`
4. **✅ RR帧自动发送** - `dlms-session/src/hdlc/connection.rs`
5. **✅ 帧重传机制** - `dlms-session/src/hdlc/window.rs`
6. **✅ 窗口管理 (SendWindow/ReceiveWindow)** - `dlms-session/src/hdlc/window.rs`
7. **✅ 服务器基础框架 (DlmsServer, CosemObject)** - `dlms-server/src/server.rs`
8. **✅ 服务器监听器 (ServerListener, ClientHandler)** - `dlms-server/src/listener.rs`
9. **✅ 认证挑战-响应流程** - `dlms-security/src/auth_flow.rs`
10. **✅ Data接口类 (Class ID: 1)** - `dlms-interface/src/data.rs`
11. **✅ Register接口类 (Class ID: 3)** - `dlms-interface/src/register.rs`
12. **✅ 访问选择器** - `dlms-application/src/selective_access.rs`
13. **✅ 协议识别服务** - `dlms-application/src/protocol_identification.rs`
14. **✅ 加密帧构建和解析** - `dlms-security/src/encryption.rs`
15. **✅ 帧计数器验证** - `dlms-security/src/encryption.rs`

---

---

## 📚 DLMS绿皮书（第八版）功能完整性评估

基于《DLMS/COSEM架构和协议第八版能源计量技术报告配套规范》的评估：

### ✅ 已实现的核心功能

#### 1. 设备语言消息规范（DLMS）
- ✅ 数据访问：GET/SET/ACTION服务完整实现
- ✅ 数据通信：PDU编码/解码完整实现
- ✅ 数据表示：DataObject及多种数据类型完整实现

#### 2. 通信模型
- ✅ 客户端/服务端模型：完整实现
- ✅ 消息类型：主要PDU类型完整实现
- ✅ 通信模式：HDLC和Wrapper协议支持

#### 3. 命名与寻址
- ✅ 逻辑名称（LN）寻址：完整实现
- ✅ 短名称（SN）寻址：完整实现
- ✅ OBIS代码：完整实现

#### 4. 物理层服务
- ✅ PH-CONNECT概念：通过TransportLayer::open()实现
- ✅ TCP/UDP/Serial传输：完整实现
- ✅ 连接管理：完整实现

#### 5. 会话层
- ✅ HDLC协议：完整实现（帧编码、FCS/HCS、窗口管理、SNRM/UA、DISC/DM）
- ✅ Wrapper协议：基础实现
- ✅ 状态机：完整实现

#### 6. 安全环境（部分）
- ✅ 加密：AES-GCM完整实现
- ✅ 安全上下文：xDLMS Context完整实现
- ✅ 基础认证：GMAC, Low-level, HLS5-GMAC实现
- ✅ 密钥管理：KDF、密钥包装（RFC 3394）实现
- ✅ 加密帧：加密帧构建和解析完整实现

#### 7. ISO-ACSE层（部分）
- ✅ AARQ/AARE：基础编码/解码实现
- ✅ RLRQ/RLRE：基础编码/解码实现
- ✅ 应用上下文：DLMS应用上下文OID定义

### ⚠️ 部分实现/待完善的功能

#### 1. 公共对象模型（COSEM）
- ⚠️ 接口类实现：仅有框架（CosemObject trait），具体接口类待实现
  - 状态：所有接口类（Data, Register, Profile Generic, Clock等）均为TODO
  - 优先级：高（影响COSEM对象模型完整性）

#### 2. 安全环境（高级功能）
- ⚠️ 认证挑战-响应流程：基础方法存在，完整流程未实现
- ⚠️ 访问控制列表（ACL）：未实现
- ⚠️ 密钥协商机制：未实现

#### 3. ISO-ACSE（高级功能）
- ⚠️ ApplicationContextNameList：未完整实现（SEQUENCE OF）
- ⚠️ AssociateSourceDiagnostic：CHOICE支持不完整
- ⚠️ APTitle/AEQualifier Form 1：仅支持Form 2
- ⚠️ AuthenticationValue：CHOICE支持不完整
- ⚠️ 认证机制OID常量：未定义
- ⚠️ ACSE Requirements位定义：未实现

### ❌ 未实现的功能

#### 1. 协议识别服务
- ✅ 状态：已实现
- ✅ 说明：已实现完整的协议识别服务，可以从InitiateResponse自动识别协议版本、特性和设备能力
- ✅ 位置：`dlms-application/src/protocol_identification.rs`

#### 2. 第三方数据交换
- ❌ 状态：未明确实现
- ❌ 说明：框架支持数据交换，但未专门针对第三方系统设计接口
- ❌ 优先级：低

#### 3. 系统集成和计量表安装相关
- ❌ 设备发现机制：未实现
- ❌ 自动配置：未实现
- ❌ 安装向导：未实现
- ❌ 优先级：低

### 📈 符合度总结

| 功能类别 | 完成度 | 说明 |
|---------|--------|------|
| DLMS核心规范 | 85% | 数据访问、通信、表示基本完整 |
| COSEM对象模型 | 15% | Data和Register接口类已实现，其他接口类待实现 |
| 通信模型 | 90% | 客户端/服务端、消息类型基本完整 |
| 命名与寻址 | 100% | LN/SN寻址完整实现 |
| 安全环境 | 80% | 加密完整，认证挑战-响应流程已实现，访问控制待完善 |
| 物理层服务 | 90% | 传输层完整，PH-CONNECT概念已实现 |
| ISO-ACSE | 60% | 基础PDU完整，高级功能待完善 |
| 协议识别 | 100% | 完整实现，支持版本、特性和能力识别 |
| 第三方数据交换 | 30% | 基础支持，专门接口未实现 |

**总体评估**：框架已具备DLMS/COSEM协议的核心通信能力，可用于基本的智能表计通信，但距离完整的商业级实现仍有差距，特别是在接口类和高级安全功能方面。

---

---

## 🔴 高优先级（核心功能）

### 1. 应用层 (dlms-application)

#### 访问选择器

- [x] **完整的访问选择器支持**
  - [x] 日期范围选择器（使用CosemDateTime）
  - [x] 入口选择器（EntryIndex）
  - [x] 值范围选择器（ValueRange）
  - [x] 与SelectiveAccessDescriptor的转换
  - [x] 辅助方法（entry_index, date_range, value_range）

### 2. 会话层 (dlms-session)

#### HDLC 高级功能

- [x] **服务器端SNRM/UA握手实现**

  - [x] 等待SNRM帧
  - [x] 解析SNRM参数
  - [x] 生成UA响应
  - [x] 发送UA帧

  - 位置: `dlms-server/src/listener.rs` (已实现`HdlcConnection::accept()`)
- [x] **请求解析和路由**

  - [x] PDU类型识别
  - [x] 路由到相应的处理方法（GET/SET/ACTION）
  - [x] 生成和发送响应

  - 位置: `dlms-server/src/listener.rs` (已实现`parse_and_route_request_hdlc`和`parse_and_route_request_wrapper`)

### 3. 安全层 (dlms-security)

#### xDLMS 加密帧

- [x] **加密帧构建和解析**

  - [x] Security Control字节处理
  - [x] System Title嵌入
  - [x] Frame Counter嵌入
  - [x] 加密数据封装
  - [x] 解密帧解析
- [x] **帧计数器验证**

  - [x] 接收帧的计数器验证（在`EncryptedFrameParser::parse_encrypted_frame`中实现）
  - [x] 重放攻击检测（检查接收计数器是否大于当前计数器）
  - [x] 计数器同步机制（自动更新接收计数器）
- [x] **KDF算法完善**

  - [x] 实现完整的DLMS标准KDF算法（符合Green Book Edition 9）
  - [x] 支持AES-128/192/256主密钥
  - [x] 使用AES-ECB模式加密（符合DLMS标准）
  - [x] 完整的文档和测试用例

---

## 🟡 中优先级（重要功能）

### 1. 服务器 (dlms-server)

#### 连接管理

- [ ] **多客户端连接管理完善**

  - [ ] 连接池管理
  - [ ] 连接状态跟踪
  - [ ] 连接超时处理
  - [ ] 优雅关闭
- [ ] **服务器状态管理**

  - [ ] 服务器状态机
  - [ ] 启动/停止流程
  - [ ] 状态转换验证

#### 请求处理

- [ ] **请求验证和授权**

  - [ ] 访问控制列表（ACL）
  - [ ] 权限验证
  - [ ] 安全策略检查
- [ ] **Get Request Next/WithList 完整支持**

  - [ ] 块传输处理
  - [ ] 多属性请求处理
  - [ ] 响应组装
- [ ] **Short Name 寻址支持**

  - [ ] base_name到OBIS码映射
  - [ ] SN寻址请求处理

#### 事件处理

- [ ] **事件通知生成**

  - [ ] 事件触发机制
  - [ ] 事件数据构建
- [ ] **事件订阅管理**

  - [ ] 订阅注册
  - [ ] 订阅列表管理
- [ ] **事件推送机制**

  - [ ] 异步事件推送
  - [ ] 推送队列管理

#### 高级功能

- [ ] **服务器统计信息**

  - [ ] 请求计数
  - [ ] 错误统计
  - [ ] 性能指标
- [ ] **并发请求处理**

  - [ ] 请求队列
  - [ ] 并发控制
  - [ ] 资源管理

### 2. 客户端 (dlms-client)

#### 高级功能

- [ ] **对象浏览功能**

  - [ ] 对象列表获取
  - [ ] 对象树遍历
  - [ ] 对象信息查询
- [ ] **数据读取功能**

  - [ ] 批量数据读取
  - [ ] 数据缓存
  - [ ] 数据格式化
- [ ] **数据写入功能**

  - [ ] 批量数据写入
  - [ ] 写入验证
  - [ ] 回滚机制
- [ ] **方法调用功能**

  - [ ] 方法参数验证
  - [ ] 返回值处理
  - [ ] 错误处理
- [ ] **事件通知处理**

  - [ ] 事件监听
  - [ ] 事件过滤
  - [ ] 事件回调

#### 连接管理

- [ ] **连接池管理**

  - [ ] 连接复用
  - [ ] 连接生命周期管理
  - [ ] 连接健康检查
- [ ] **自动重连机制**

  - [ ] 连接断开检测
  - [ ] 自动重连策略
  - [ ] 重连次数限制
- [ ] **请求/响应超时处理**

  - [ ] 可配置超时时间
  - [ ] 超时重试机制
  - [ ] 超时回调
- [ ] **并发请求支持**

  - [ ] 请求队列
  - [ ] 并发控制
  - [ ] 请求去重
- [ ] **请求队列管理**

  - [ ] 优先级队列
  - [ ] 队列大小限制
  - [ ] 队列监控
- [ ] **客户端配置管理**

  - [ ] 配置文件支持
  - [ ] 配置验证
  - [ ] 配置热重载

### 3. 安全层 (dlms-security)

#### 认证功能

- [x] **认证挑战-响应流程**

  - [x] 挑战生成（随机挑战生成，可配置长度）
  - [x] 响应验证（Low-level和HLS5-GMAC支持）
  - [x] 认证状态管理（AuthenticationState状态机）
  - [x] 挑战超时机制
  - [x] 多种认证机制支持（Low-level, HLS5-GMAC, GMAC）
  - [x] 完整的单元测试
  - 位置: `dlms-security/src/auth_flow.rs`
- [ ] **密钥协商机制**

  - [ ] 密钥交换协议
  - [ ] 密钥验证
  - [ ] 密钥更新
- [ ] **认证状态管理**

  - [ ] 认证状态跟踪
  - [ ] 状态转换
  - [ ] 状态验证

#### 密钥管理

- [ ] **密钥存储和管理**

  - [ ] 密钥存储接口
  - [ ] 密钥加密存储
  - [ ] 密钥访问控制
- [ ] **密钥更新机制**

  - [ ] 密钥轮换
  - [ ] 密钥同步
  - [ ] 密钥撤销
- [ ] **主密钥（KEK）管理**

  - [ ] KEK生成
  - [ ] KEK分发
  - [ ] KEK更新
- [ ] **密钥导出和导入**

  - [ ] 密钥导出格式
  - [ ] 密钥导入验证
  - [ ] 密钥迁移

#### 安全套件

- [ ] **安全套件协商**

  - [ ] 套件列表交换
  - [ ] 套件选择算法
  - [ ] 套件验证
- [ ] **安全参数验证**

  - [ ] 参数完整性检查
  - [ ] 参数范围验证
  - [ ] 参数兼容性检查

### 4. 协议识别服务 (dlms-application)

#### 协议识别功能

- [x] **协议识别服务基础框架**

  - [x] ProtocolIdentification结构体
  - [x] ProtocolInfo结构体
  - [x] 从InitiateResponse提取协议信息
- [x] **协议版本识别**

  - [x] DLMS版本检测
  - [x] 版本验证
- [x] **协议特性识别**

  - [x] Conformance bits解析
  - [x] 服务支持检测（GET/SET/ACTION等）
  - [x] 功能支持检测（SelectiveAccess、BlockRead等）
- [x] **设备能力识别**

  - [x] PDU大小识别
  - [x] 支持的服务列表
  - [x] 支持的功能列表
  - [x] 能力描述生成
- [x] **协议识别服务集成**

  - [x] 添加到dlms-application模块
  - [x] 导出ProtocolIdentification和ProtocolInfo
  - [x] 单元测试

  - 位置: `dlms-application/src/protocol_identification.rs`

### 5. ISO-ACSE (dlms-asn1)

#### 高级功能

- [ ] **ApplicationContextNameList 完整实现**

  - [ ] SEQUENCE OF编码/解码
  - [ ] 列表验证
- [ ] **AssociateSourceDiagnostic 完整 CHOICE 支持**

  - [ ] 所有CHOICE变体
  - [ ] 变体编码/解码
- [ ] **AcseServiceUser 和 AcseServiceProvider 枚举**

  - [ ] 枚举定义
  - [ ] 编码/解码
- [ ] **APTitle Form 1 支持**

  - [ ] Form 1编码/解码
  - [ ] Form 1/2转换
- [ ] **AEQualifier Form 1 支持**

  - [ ] Form 1编码/解码
  - [ ] Form 1/2转换
- [ ] **AuthenticationValue 完整 CHOICE 支持**

  - [ ] 所有CHOICE变体（null-data, boolean, bit-string, double-long等，共33种数据类型）
  - [ ] 变体编码/解码（BER格式）
  - [ ] 类型安全的枚举表示所有CHOICE变体
  - [ ] 保持向后兼容性（现有OCTET STRING支持必须继续工作）
  - [ ] 参考：DLMS绿皮书定义的完整CHOICE结构
  - 注意：当前实现仅支持OCTET STRING（最常用），扩展时需确保不破坏现有功能
- [ ] **常用认证机制 OID 常量**

  - [ ] OID常量定义
  - [ ] OID验证函数
- [ ] **ACSE Requirements 位定义**

  - [ ] 位定义常量
  - [ ] 位操作函数
- [ ] **ACSE 错误处理增强**

  - [ ] 详细错误信息
  - [ ] 错误恢复机制

---

## 🟢 低优先级（增强功能）

### 1. 会话层 (dlms-session)

#### HDLC 优化

- [ ] **HDLC 错误恢复机制**
  - [ ] 错误检测
  - [ ] 自动恢复
  - [ ] 错误报告

#### Wrapper 优化

- [ ] **Wrapper 连接建立流程**

  - [ ] 连接握手
  - [ ] 参数协商
- [ ] **Wrapper 错误处理**

  - [ ] 错误检测
  - [ ] 错误恢复

#### 通用功能

- [ ] **会话状态管理**

  - [ ] 状态机实现
  - [ ] 状态转换验证
- [ ] **多会话支持**

  - [ ] 会话标识
  - [ ] 会话管理
  - [ ] 会话隔离

### 2. 传输层 (dlms-transport)

#### TCP 优化

- [ ] **连接池管理**

  - [ ] 连接复用
  - [ ] 连接生命周期
- [ ] **自动重连机制**

  - [ ] 重连策略
  - [ ] 重连次数限制
- [ ] **超时处理优化**

  - [ ] 可配置超时
  - [ ] 超时回调

#### UDP 优化

- [ ] **数据包分片和重组**

  - [ ] 分片处理
  - [ ] 重组逻辑
- [ ] **数据包丢失检测**

  - [ ] 丢失检测
  - [ ] 重传机制

#### Serial 优化

- [ ] **串口参数自动检测**

  - [ ] 波特率检测
  - [ ] 数据位检测
- [ ] **流控制支持**

  - [ ] RTS/CTS支持
  - [ ] XON/XOFF支持
- [ ] **多串口设备管理**

  - [ ] 设备枚举
  - [ ] 设备选择

#### 通用功能

- [ ] **传输层统计信息**

  - [ ] 字节计数
  - [ ] 错误统计
  - [ ] 性能指标
- [ ] **连接状态监控**

  - [ ] 状态跟踪
  - [ ] 状态通知
- [ ] **错误恢复机制**

  - [ ] 自动恢复
  - [ ] 错误报告

### 3. ASN.1 (dlms-asn1)

#### A-XDR 优化

- [ ] **CompactArray 完整编码/解码支持**

  - [ ] 编码实现
  - [ ] 解码实现
  - [ ] 测试覆盖
- [ ] **长度编码优化（支持长格式）**

  - [ ] 长格式支持
  - [ ] 编码优化
- [ ] **错误处理和恢复机制**

  - [ ] 错误检测
  - [ ] 部分解码
  - [ ] 错误恢复

#### COSEM ASN.1

- [ ] **生成 COSEM ASN.1 结构定义**

  - [ ] 结构生成工具
  - [ ] 结构定义
- [ ] **实现 COSEM 对象标识符编码/解码**

  - [ ] OID编码
  - [ ] OID解码
- [ ] **实现 COSEM 方法调用编码/解码**

  - [ ] 方法编码
  - [ ] 方法解码
- [ ] **实现 COSEM 属性访问编码/解码**

  - [ ] 属性编码
  - [ ] 属性解码

### 4. 接口类 (dlms-interface)

#### 核心接口类

- [x] **Data 接口类（Class ID: 1）**
  - [x] 属性1：logical_name（OBIS代码）
  - [x] 属性2：value（数据值）
  - [x] CosemObject trait实现
  - [x] 单元测试
  - 位置: `dlms-interface/src/data.rs`
- [x] **Register 接口类（Class ID: 3）**
  - [x] 属性1：logical_name（OBIS代码）
  - [x] 属性2：value（寄存器值）
  - [x] 属性3：scaler_unit（量纲和单位）
  - [x] 属性4：status（可选状态值）
  - [x] ScalerUnit类型实现（缩放因子和单位代码）
  - [x] CosemObject trait实现
  - [x] 缩放值计算功能
  - [x] 单元测试
  - 位置: `dlms-interface/src/register.rs`, `dlms-interface/src/scaler_unit.rs`
- [ ] **Extended Register 接口类（Class ID: 4）**
- [ ] **Demand Register 接口类（Class ID: 5）**
- [ ] **Profile Generic 接口类（Class ID: 7）**
- [ ] **Clock 接口类（Class ID: 8）**
- [ ] **Association Short Name 接口类（Class ID: 12）**
- [ ] **Association Logical Name 接口类（Class ID: 15）**
- [ ] **Security Setup 接口类（Class ID: 64）**

#### 其他接口类

- [ ] Register Activation（Class ID: 6）
- [ ] Script Table（Class ID: 9）
- [ ] Schedule（Class ID: 10）
- [ ] Special Days Table（Class ID: 11）
- [ ] SAP Assignment（Class ID: 17）
- [ ] Image Transfer（Class ID: 18）
- [ ] IEC Local Port Setup（Class ID: 19）
- [ ] Activity Calendar（Class ID: 20）
- [ ] Register Monitor（Class ID: 21）
- [ ] Single Action Schedule（Class ID: 22）
- [ ] IEC HDLC Setup（Class ID: 23）
- [ ] IEC twisted pair setup（Class ID: 24）
- [ ] MBus Slave Port Setup（Class ID: 25）
- [ ] Disconnect Control（Class ID: 70）
- [ ] Limiter（Class ID: 71）
- [ ] Push Setup（Class ID: 40）

#### 接口类基础设施

- [ ] **属性处理**

  - [ ] 属性访问器实现
  - [ ] 属性值验证
  - [ ] 属性访问权限检查
- [ ] **方法处理**

  - [ ] 方法调用实现
  - [ ] 方法参数验证
  - [ ] 方法返回值处理
- [ ] **宏系统**

  - [ ] 接口类定义宏
  - [ ] 属性定义宏
  - [ ] 方法定义宏

### 5. 核心模块 (dlms-core)

#### 数据类型

- [ ] **完善数据类型单元测试**

  - [ ] 所有数据类型测试
  - [ ] 边界条件测试
  - [ ] 错误情况测试
- [ ] **实现更多 COSEM 日期/时间格式支持**

  - [ ] 其他日期格式
  - [ ] 时区支持
  - [ ] 夏令时支持
- [ ] **添加数据类型验证和约束检查**

  - [ ] 范围验证
  - [ ] 格式验证
  - [ ] 约束检查
- [ ] **实现 OBIS 代码解析和验证工具**

  - [ ] OBIS解析器
  - [ ] OBIS验证器
  - [ ] OBIS格式化
- [ ] **添加数据类型转换工具函数**

  - [ ] 类型转换
  - [ ] 格式化函数
  - [ ] 解析函数

---

## 📝 优化和增强

### 性能优化

- [ ] **内存优化**

  - [ ] 零拷贝操作
  - [ ] 内存池
  - [ ] 缓冲区复用
- [ ] **编码/解码优化**

  - [ ] 编码缓存
  - [ ] 批量操作
  - [ ] SIMD优化
- [ ] **并发优化**

  - [ ] 无锁数据结构
  - [ ] 并发控制优化
  - [ ] 资源池管理

### 代码质量

- [ ] **测试覆盖**

  - [ ] 单元测试
  - [ ] 集成测试
  - [ ] 性能测试
  - [ ] 模糊测试
- [ ] **文档完善**

  - [ ] API文档
  - [ ] 使用示例
  - [ ] 架构文档
  - [ ] 协议文档
- [ ] **错误处理增强**

  - [ ] 详细错误信息
  - [ ] 错误恢复机制
  - [ ] 错误日志

### 工具和基础设施

- [ ] **开发工具**

  - [ ] 调试工具
  - [ ] 性能分析工具
  - [ ] 协议分析工具
- [ ] **CI/CD**

  - [ ] 自动化测试
  - [ ] 代码质量检查
  - [ ] 自动化构建
  - [ ] 发布流程
- [ ] **示例和教程**

  - [ ] 基础示例
  - [ ] 高级示例
  - [ ] 最佳实践
  - [ ] 故障排除指南

---

## 📅 优先级说明

- **🔴 高优先级**: 核心功能，影响基本使用
- **🟡 中优先级**: 重要功能，提升用户体验
- **🟢 低优先级**: 增强功能，优化和扩展

---

## 📌 近期重点（Next Sprint）

### ✅ 已完成（2025-01-15之前）
1. ✅ **服务器端SNRM/UA握手实现** - `dlms-server/src/listener.rs`
2. ✅ **请求解析和路由** - `dlms-server/src/listener.rs`
3. ✅ **加密帧构建和解析** - `dlms-security/src/encryption.rs`
4. ✅ **帧计数器验证** - `dlms-security/src/encryption.rs`
5. ✅ **完整的访问选择器支持** - `dlms-application/src/selective_access.rs`
6. ✅ **协议识别服务** - `dlms-application/src/protocol_identification.rs`
7. ✅ **认证挑战-响应流程** - `dlms-security/src/auth_flow.rs`
8. ✅ **Data接口类实现** - `dlms-interface/src/data.rs`
9. ✅ **Register接口类实现** - `dlms-interface/src/register.rs`
10. ✅ **窗口管理和帧重传** - `dlms-session/src/hdlc/window.rs`

### ⏳ 当前Sprint（2025-01-15开始）

#### 🔴 最高优先级
1. **修复Conformance编码方式**（BER编码，不是A-XDR）
2. **实现COSEM-OPEN服务原语**（CosemOpenRequest/Confirm）
3. **实现COSEM-RELEASE服务原语**（CosemReleaseRequest/Confirm/Abort）
4. **实现AARQ/AARE封装**（InitiateRequest插入user_Information域）

#### 🟡 高优先级
5. **实现加密PDU支持**（glo-*/ded-*，34种PDU）
6. **实现SN寻址PDU**（ReadRequest/WriteRequest等，6种PDU）
7. **实现ConfirmedServiceError**

#### 🟢 中优先级
8. **完善LLC Header处理**
9. **实现ISO-ACSE高级功能**
10. **实现Data-Notification和InformationReportRequest**
11. **COSEM接口类**（Profile Generic, Clock, Extended Register等）

---

## 📊 统计信息

- **总待办事项**: ~150项
- **高优先级**: ~15项
- **中优先级**: ~60项
- **低优先级**: ~75项

---

## 🔄 更新历史

- 2026-01-15: Bug修复完成
  - 修复所有8个Bug:
    - Bug #1: ObisCode Display格式字符串添加缺少的点号
    - Bug #2: read_exact函数移入impl块内部
    - Bug #3: HDLC Window wrap-around逻辑修复，添加旧周期帧确认
    - Bug #4: TCP timeout应用于connect调用
    - Bug #5: CosemDate测试用例修复，移除无效的溢出测试
    - Bug #6: 移除deprecated closed字段，统一使用state
    - Bug #7: 移除不必要的mut绑定
    - Bug #8: 移除冗余的closed状态检查
- 2026-01-15: Bug审查和TODO更新
  - 完成代码库静态分析，发现11个问题（2严重/3中等/3轻微/3改进）
  - 更新Bug报告: `dlms-docs/bugreview/BUG_REPORT.md`
  - 在TODO中添加Bug修复跟踪章节
- 2025-01-15: 整合dlms-docs需求并更新TODO
  - 阅读并整理`dlms-docs/`目录下所有需求文档
  - 添加"来自dlms-docs的需求汇总"章节
  - 更新符合度总览和最高优先级需求
  - 更新近期重点Sprint计划
  - 详细文档位置：
    - `dlms-docs/requirements/REQUIREMENTS_SUMMARY.md`
    - `dlms-docs/requirements/COSEM_PDU_ASN1_REQUIREMENTS.md`
    - `dlms-docs/requirements/COSEM_RELEASE_REQUIREMENTS.md`
    - `dlms-docs/COSEM_OPEN_ANALYSIS.md`
    - `dlms-docs/IMPLEMENTATION_COMPLIANCE_CHECK.md`
    - `dlms-docs/IMPLEMENTATION_COMPARISON_REPORT.md`
    - `dlms-docs/SERVER_IMPLEMENTATION_SUMMARY.md`
    - `dlms-docs/WINDOW_MANAGEMENT_IMPLEMENTATION.md`
    - `dlms-docs/LISTENER_IMPLEMENTATION_SUMMARY.md`
    - `dlms-docs/COSEM_LIBRARY_IMPROVEMENTS.md`
- 2025-01-14: COSEM PDU ASN.1定义符合度分析
  - 创建COSEM_PDU_ASN1_REQUIREMENTS.md文档
  - 分析COSEMpdu ASN.1定义的完整符合度
  - 发现关键问题：Conformance应使用BER编码（不是A-XDR）
  - 发现缺失：加密PDU（glo-*/ded-*）、SN寻址PDU、ConfirmedServiceError等
  - 当前符合度：约50%（非加密LN寻址100%，其他0%）
- 2025-01-14: 创建需求整理汇总文档
  - 创建REQUIREMENTS_SUMMARY.md汇总所有需求
  - 包含术语实现、COSEM-OPEN、COSEM-RELEASE、SHA-1等所有需求
  - 提供符合度评估和实现计划
  - 位置: `dlms-docs/requirements/REQUIREMENTS_SUMMARY.md`
- 2025-01-14: SHA-1算法实现需求分析
  - 创建SHA1_ALGORITHM_REQUIREMENTS.md文档
  - 分析SHA-1算法完整规范（消息填充、长度附加、主循环等）
  - 整理实现需求（待确认DLMS/COSEM是否要求SHA-1）
  - 当前状态：未实现（0%），需要先确认协议要求
- 2025-01-14: COSEM-RELEASE服务规范需求分析
  - 创建COSEM_RELEASE_REQUIREMENTS.md文档
  - 分析COSEM-RELEASE.request、COSEM-RELEASE.confirm和COSEM-ABORT.indication符合度
  - 整理实现需求（高/中/低优先级）
  - 当前符合度：约40%（request 70%, confirm 40%, abort 10%）
- 2025-01-14: COSEM-OPEN服务规范符合度分析
  - 创建COSEM_OPEN_ANALYSIS.md文档
  - 分析COSEM-OPEN.request和COSEM-OPEN.confirm符合度
  - 整理实现需求（高/中/低优先级）
  - 当前符合度：约50%（request 60%, confirm 40%）
- 2025-01-14: 实现认证挑战-响应流程（AuthenticationFlow）
  - 完成挑战生成和状态管理
  - 完成响应验证和认证状态转换
  - 支持多种认证机制（Low-level, HLS5-GMAC）
  - 实现挑战超时机制
  - 添加完整的单元测试
- 2025-01-14: 实现Register接口类（Class ID: 3）
  - 完成所有4个属性的实现（logical_name, value, scaler_unit, status）
  - 实现ScalerUnit类型（缩放因子和单位代码支持）
  - 完成CosemObject trait实现
  - 添加缩放值计算功能
  - 添加完整的单元测试
- 2025-01-14: 实现Data接口类（Class ID: 1）
  - 完成属性1（logical_name）和属性2（value）实现
  - 完成CosemObject trait实现
  - 添加完整的单元测试
- 2025-01-14: 添加DLMS绿皮书功能完整性评估章节
- 2025-01-14: 实现协议识别服务（ProtocolIdentification）
  - 完成协议版本识别
  - 完成协议特性识别（Conformance bits解析）
  - 完成设备能力识别
  - 添加完整的单元测试
- 2025-01-XX: 初始TODO清单创建
- 定期更新: 根据开发进度更新状态
