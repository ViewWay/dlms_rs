# Get Request/Response WithList PDU 实现方案

## 概述

实现 `GetRequest::WithList` 和 `GetResponse::WithList` 的完整编码/解码功能。这两个PDU用于批量读取多个属性，提高通信效率。

## 当前状态

### 已完成
- ✅ `GetRequest::Normal` 和 `GetResponse::Normal` 完整实现
- ✅ `GetRequest::Next` 和 `GetResponse::WithDataBlock` 基本实现（需要验证）
- ✅ `CosemAttributeDescriptor`、`SelectiveAccessDescriptor`、`GetDataResult` 等基础类型已实现

### 待实现
- ❌ `GetRequest::WithList` 编码/解码
- ❌ `GetResponse::WithList` 编码/解码

## 数据结构分析

### GetRequest::WithList

```rust
WithList {
    invoke_id_and_priority: InvokeIdAndPriority,
    attribute_descriptor_list: Vec<CosemAttributeDescriptor>,
    access_selection_list: Option<Vec<Option<SelectiveAccessDescriptor>>>,
}
```

**字段说明：**
- `invoke_id_and_priority`: 调用ID和优先级（8位BitString）
- `attribute_descriptor_list`: 属性描述符列表（必需，非空）
- `access_selection_list`: 选择性访问描述符列表（可选）
  - 如果提供，长度必须与 `attribute_descriptor_list` 相同
  - 每个元素也是可选的（对应属性可能不需要选择性访问）

### GetResponse::WithList

```rust
WithList {
    invoke_id_and_priority: InvokeIdAndPriority,
    result_list: Vec<GetDataResult>,
}
```

**字段说明：**
- `invoke_id_and_priority`: 调用ID和优先级（8位BitString）
- `result_list`: 结果列表（必需，非空）
  - 长度必须与请求中的 `attribute_descriptor_list` 相同
  - 每个元素是 `GetDataResult`（CHOICE类型：Data 或 DataAccessResult）

## A-XDR 编码规则

### 数组编码规则

根据 IEC 62056-47 标准，A-XDR 数组编码格式：
1. **长度编码**：使用 `LengthEncoding`（短格式 < 128，长格式 >= 128）
2. **元素编码**：依次编码每个元素，无分隔符
3. **编码顺序**：在 CHOICE 类型中，数组字段按逆序编码（最后字段在前）

### 可选字段编码规则

1. **外层可选字段**（`access_selection_list`）：
   - 先编码 `bool` 标志（true = 存在，false = 不存在）
   - 如果标志为 `true`，再编码数组

2. **内层可选字段**（`Vec<Option<SelectiveAccessDescriptor>>`）：
   - 数组中的每个元素如果是 `Option<T>`，需要：
     - 先编码 `bool` 标志
     - 如果标志为 `true`，再编码 `T` 的值

### CHOICE 类型编码规则

`GetRequest` 和 `GetResponse` 是 CHOICE 类型：
1. 先编码值（按逆序）
2. 最后编码选择标签（1=Normal, 2=Next, 3=WithList）

## 编码实现方案

### GetRequest::WithList 编码

**编码顺序（A-XDR 逆序）：**
1. `access_selection_list`（可选数组）
   - 如果存在：
     - 编码数组长度
     - 对每个元素：
       - 编码 `bool` 标志（是否存在）
       - 如果存在，编码 `SelectiveAccessDescriptor`
2. `attribute_descriptor_list`（必需数组）
   - 编码数组长度
   - 对每个元素编码 `CosemAttributeDescriptor`
3. `invoke_id_and_priority`（BitString）
   - 编码 `InvokeIdAndPriority`
4. 选择标签：`3`（WithList）

**伪代码：**
```rust
// 逆序编码
if let Some(ref access_list) = access_selection_list {
    // 编码数组长度
    encoder.encode_length(access_list.len())?;
    // 逆序编码每个元素（数组内部也是逆序）
    for access in access_list.iter().rev() {
        encoder.encode_bool(access.is_some())?;
        if let Some(ref desc) = access {
            desc.encode(&mut encoder)?;
        }
    }
} else {
    encoder.encode_bool(false)?; // 外层可选标志
}

// 编码属性描述符列表（逆序）
encoder.encode_length(attribute_descriptor_list.len())?;
for desc in attribute_descriptor_list.iter().rev() {
    desc.encode(&mut encoder)?;
}

// 编码 invoke_id_and_priority
invoke_id_and_priority.encode(&mut encoder)?;

// 编码选择标签
encoder.encode_u8(3)?;
```

### GetResponse::WithList 编码

**编码顺序（A-XDR 逆序）：**
1. `result_list`（必需数组）
   - 编码数组长度
   - 对每个元素编码 `GetDataResult`（逆序）
2. `invoke_id_and_priority`（BitString）
   - 编码 `InvokeIdAndPriority`
3. 选择标签：`3`（WithList）

**伪代码：**
```rust
// 逆序编码
// 编码结果列表（逆序）
encoder.encode_length(result_list.len())?;
for result in result_list.iter().rev() {
    result.encode(&mut encoder)?;
}

// 编码 invoke_id_and_priority
invoke_id_and_priority.encode(&mut encoder)?;

// 编码选择标签
encoder.encode_u8(3)?;
```

## 解码实现方案

### GetRequest::WithList 解码

**解码顺序（A-XDR 逆序，从后往前读）：**
1. 读取选择标签（应该是 `3`）
2. 解码 `invoke_id_and_priority`
3. 解码 `attribute_descriptor_list`
   - 读取数组长度
   - 依次解码每个 `CosemAttributeDescriptor`
4. 解码 `access_selection_list`（可选）
   - 读取 `bool` 标志
   - 如果为 `true`：
     - 读取数组长度
     - 依次解码每个元素（每个元素也是可选的）

**伪代码：**
```rust
// 从后往前解码
let choice_tag = decoder.decode_u8()?; // 应该是 3
if choice_tag != 3 {
    return Err(...);
}

let invoke_id_and_priority = InvokeIdAndPriority::decode(&mut decoder)?;

// 解码属性描述符列表
let attr_list_len = decoder.decode_length()?;
let mut attribute_descriptor_list = Vec::with_capacity(attr_list_len);
for _ in 0..attr_list_len {
    attribute_descriptor_list.push(CosemAttributeDescriptor::decode(&mut decoder)?);
}
// 注意：解码顺序与编码顺序相反，需要反转
attribute_descriptor_list.reverse();

// 解码选择性访问列表（可选）
let has_access_list = decoder.decode_bool()?;
let access_selection_list = if has_access_list {
    let access_list_len = decoder.decode_length()?;
    let mut access_list = Vec::with_capacity(access_list_len);
    for _ in 0..access_list_len {
        let has_access = decoder.decode_bool()?;
        let access = if has_access {
            Some(SelectiveAccessDescriptor::decode(&mut decoder)?)
        } else {
            None
        };
        access_list.push(access);
    }
    // 反转顺序
    access_list.reverse();
    Some(access_list)
} else {
    None
};

Ok(GetRequest::WithList { ... })
```

### GetResponse::WithList 解码

**解码顺序（A-XDR 逆序）：**
1. 读取选择标签（应该是 `3`）
2. 解码 `invoke_id_and_priority`
3. 解码 `result_list`
   - 读取数组长度
   - 依次解码每个 `GetDataResult`

## 实现细节

### 1. 数组长度编码

使用 `LengthEncoding`：
- 长度 < 128：短格式（1字节）
- 长度 >= 128：长格式（4字节：0x80 + 3字节长度）

### 2. 逆序编码处理

**重要**：A-XDR 中，SEQUENCE 字段按逆序编码，但数组内部元素的顺序是**正序**的（不是逆序）。

这意味着：
- 数组长度先编码
- 数组元素按索引 0, 1, 2, ... 顺序编码
- 但整个数组作为 SEQUENCE 的一个字段，在 SEQUENCE 中是逆序的

### 3. 错误处理

需要验证：
- 数组长度不能为 0（至少需要一个属性）
- `access_selection_list` 如果存在，长度必须与 `attribute_descriptor_list` 相同
- `result_list` 长度必须与请求的 `attribute_descriptor_list` 相同（在服务层验证）

### 4. 边界情况

- 空数组：不允许（至少需要一个属性）
- 单个属性：允许，但使用 `Normal` 类型更高效
- 大量属性：需要考虑 PDU 大小限制

## 测试方案

### 单元测试

1. **GetRequest::WithList 编码/解码测试**
   - 单个属性（带选择性访问）
   - 多个属性（部分带选择性访问）
   - 多个属性（全部带选择性访问）
   - 多个属性（无选择性访问）

2. **GetResponse::WithList 编码/解码测试**
   - 单个结果（成功）
   - 多个结果（混合成功和失败）
   - 所有结果成功
   - 所有结果失败

3. **边界测试**
   - 空数组（应该失败）
   - 数组长度验证
   - 无效的选择标签

### 集成测试

- 完整的 GET WithList 请求/响应流程
- 与 Normal 类型的对比测试
- 性能测试（大量属性）

## 优化方向

1. **内存优化**：使用 `SmallVec` 或预分配容量
2. **零拷贝**：对于大数组，考虑使用引用
3. **验证优化**：延迟验证到编码/解码时
4. **缓存**：对于频繁使用的描述符，考虑缓存编码结果

## 实现步骤

1. ✅ 设计方案（本文档）
2. ⏳ 实现 `GetRequest::WithList` 编码
3. ⏳ 实现 `GetRequest::WithList` 解码
4. ⏳ 实现 `GetResponse::WithList` 编码
5. ⏳ 实现 `GetResponse::WithList` 解码
6. ⏳ 添加单元测试
7. ⏳ 验证与 Java jDLMS 的兼容性
8. ⏳ 更新 TODO 列表

## 参考

- IEC 62056-47: DLMS/COSEM application layer
- Java jDLMS 实现：`/Users/yimiliya/IdeaProjects/jdlms`
- 当前实现：`dlms-application/src/pdu.rs`
