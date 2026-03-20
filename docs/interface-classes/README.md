# COSEM 接口类完整规范索引

> 基于 Blue Book Edition 16 Part 2 | 共 30 个接口类

---

## 🔴 高优先级（核心功能）

| Class ID | 名称 | 文件 | 属性 | 方法 | 大小 |
|----------|------|------|------|------|------|
| 1 | Data | [IC1_Data.md](IC1_Data.md) | 2 | 0 | 4KB |
| 3 | Register | [IC3_Register.md](IC3_Register.md) | 3 | 1 | 6KB |
| 7 | Profile Generic | [IC7_ProfileGeneric.md](IC7_ProfileGeneric.md) | 8 | 4 | 12KB |
| 8 | Clock | [IC8_Clock.md](IC8_Clock.md) | 9 | 6 | 12KB |
| 15 | Association LN | [IC15_AssociationLN.md](IC15_AssociationLN.md) | 11 | 6 | 16KB |
| 18 | Image Transfer | [IC18_ImageTransfer.md](IC18_ImageTransfer.md) | 7 | 4 | 12KB |
| 64 | Security Setup | [IC64_SecuritySetup.md](IC64_SecuritySetup.md) | 9 | 0 | 8KB |

---

## 🟡 中优先级（重要功能）

| Class ID | 名称 | 文件 | 属性 | 方法 |
|----------|------|------|------|------|
| 4 | Extended Register | [IC4_ExtendedRegister.md](IC4_ExtendedRegister.md) | 5 | 1 |
| 5 | Demand Register | [IC5_DemandRegister.md](IC5_DemandRegister.md) | 9 | 2 |
| 6 | Register Activation | [IC6_RegisterActivation.md](IC6_RegisterActivation.md) | 3 | 2 |
| 9 | Script Table | [IC9_ScriptTable.md](IC9_ScriptTable.md) | 2 | 1 |
| 10 | Schedule | [IC10_Schedule.md](IC10_Schedule.md) | 2 | 0 |
| 11 | Special Days Table | [IC11_SpecialDaysTable.md](IC11_SpecialDaysTable.md) | 2 | 0 |
| 12 | Association SN | [IC12_AssociationSN.md](IC12_AssociationSN.md) | 3 | 0 |
| 20 | Activity Calendar | [IC20_ActivityCalendar.md](IC20_ActivityCalendar.md) | 9 | 0 |
| 21 | Register Monitor | [IC21_RegisterMonitor.md](IC21_RegisterMonitor.md) | 4 | 0 |
| 22 | Single Action Schedule | [IC22_SingleActionSchedule.md](IC22_SingleActionSchedule.md) | 2 | 0 |
| 40 | Push Setup | [IC40_PushSetup.md](IC40_PushSetup.md) | 9 | 0 |
| 70 | Disconnect Control | [IC70_DisconnectControl.md](IC70_DisconnectControl.md) | 4 | 2 |
| 71 | Limiter | [IC71_Limiter.md](IC71_Limiter.md) | 9 | 0 |

---

## 🟢 低优先级（配置类）

| Class ID | 名称 | 文件 | 属性 | 方法 |
|----------|------|------|------|------|
| 17 | SAP Assignment | [IC17_SAPAssignment.md](IC17_SAPAssignment.md) | 2 | 0 |
| 19 | IEC Local Port Setup | [IC19_IECLocalPortSetup.md](IC19_IECLocalPortSetup.md) | 5 | 0 |
| 23 | IEC HDLC Setup | [IC23_IECHDLCSetup.md](IC23_IECHDLCSetup.md) | 9 | 0 |
| 27 | Modem Configuration | [IC27_ModemConfiguration.md](IC27_ModemConfiguration.md) | 5 | 0 |
| 28 | Auto Answer | [IC28_AutoAnswer.md](IC28_AutoAnswer.md) | 6 | 1 |
| 29 | Auto Connect | [IC29_AutoConnect.md](IC29_AutoConnect.md) | 6 | 1 |
| 41 | TCP-UDP Setup | [IC41_TCPUDPSetup.md](IC41_TCPUDPSetup.md) | 8 | 0 |
| 42 | IPv4 Setup | [IC42_IPv4Setup.md](IC42_IPv4Setup.md) | 10 | 0 |
| 43 | MAC Address Setup | [IC43_MACAddressSetup.md](IC43_MACAddressSetup.md) | 2 | 0 |
| 48 | IPv6 Setup | [IC48_IPv6Setup.md](IC48_IPv6Setup.md) | 4 | 0 |

---

## 规范文档结构

每个完整规范包含：

1. **概述** - 接口类用途和功能说明
2. **属性定义** - 属性表格 + 详细说明
3. **方法定义** - 方法表格 + 详细说明
4. **Rust 完整实现** - 数据结构、trait 实现、方法
5. **测试用例** - 单元测试代码
6. **实现检查清单** - 完成度跟踪
7. **相关文档** - 参考资料

---

## 统计

| 分类 | 数量 | 总属性 | 总方法 |
|------|------|--------|--------|
| 🔴 高优先级 | 7 | 49 | 17 |
| 🟡 中优先级 | 13 | 59 | 4 |
| 🟢 低优先级 | 10 | 57 | 2 |
| **总计** | **30** | **165** | **23** |

---

## 快速参考

### 文件命名规则

```
IC{class_id}_{ClassName}.md
```

### 最常用 OBIS 码

| OBIS 码 | 接口类 | 说明 |
|---------|--------|------|
| 0.0.40.0.0.255 | IC15 | 当前关联 |
| 0.0.1.0.0.255 | IC8 | 设备时钟 |
| 1.0.1.8.0.255 | IC3 | 总有功电能 |
| 0.0.43.0.0.255 | IC64 | 安全设置 |

### Short Name 计算

**属性**: `base_name + (attr_id - 1) * 8`
**方法**: `base_name + 0x60 + (method_id - 1) * 8`

---

*更新时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
*生成器: generate_complete_v3.py*
