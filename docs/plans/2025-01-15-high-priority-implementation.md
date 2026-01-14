# DLMS/COSEM High Priority Requirements Implementation Plan

**Created**: 2025-01-15
**Status**: Design Complete, Implementation In Progress

---

## Overview

This document covers the implementation of 5 high-priority requirements for DLMS/COSEM protocol compliance:

1. COSEM-OPEN/RELEASE Service Primitives
2. AARQ/AARE Encapsulation
3. Encrypted PDU (glo-*/ded-*)
4. Short Name (SN) Addressing PDU

---

## Phase 1: Association Module

### Architecture

```
dlms-application/src/association/
├── mod.rs          # Public exports
├── state.rs        # State machine definition
├── context.rs      # AssociationContext structure
└── events.rs       # Association events
```

### State Machine

```rust
pub enum AssociationState {
    Inactive,           // No association
    Idle,               // Physical connection established
    AssociationPending,  // AARQ sent, waiting for AARE
    Associated,         // Application association established
    ReleasePending,     // RLRQ sent, waiting for RLRE
}
```

### Service Primitives Mapping

| DLMS Primitive | Method | Returns |
|----------------|--------|---------|
| COSEM-OPEN.request | `Association::open()` | `OpenResult` |
| COSEM-RELEASE.request | `Association::release()` | `ReleaseResult` |
| COSEM-ABORT.indication | `Association::abort()` | `AbortReason` |

---

## Phase 2: AARQ/AARE Encapsulation

### AARQ Structure

```
AARQ APDU:
├── application-context-name (OID)
├── called-AP-title (optional)
├── called-AP-invocation-identifier (optional)
├── calling-AP-title (optional)
├── calling-AP-invocation-identifier (optional)
├── authentication-value (optional)
└── user-Information (InitiateRequest) ← Key
```

### Connection Flow

```
Client                    Server
  |                         |
  |-- SNRM (HDLC) -------->|
  |<-- UA (HDLC) ----------|
  |                         |
  |-- AARQ + InitiateReq -->|  (COSEM-OPEN.request)
  |                         |
  |<-- AARE + InitiateRes ---|  (COSEM-OPEN.confirm)
  |                         |
  |    Associated           |
```

---

## Phase 3: Encrypted PDU

### PDU Types

| Type | Prefix | Count | Purpose |
|------|--------|-------|---------|
| Global Encryption | glo- | 17 | Global cipher key |
| Dedicated Encryption | ded- | 17 | Dedicated cipher key |

### Encryption Format

```
Security Control (1 byte)
System Title (8 bytes)
Frame Counter (4 bytes)
Encrypted Data (variable)
Authentication Tag (12 bytes for GMAC)
```

---

## Phase 4: SN Addressing PDU

### PDU Mapping

| SN PDU | LN PDU Equivalent |
|--------|------------------|
| ReadRequest | GetRequest |
| ReadResponse | GetResponse |
| WriteRequest | SetRequest |
| WriteResponse | SetResponse |
| UnconfirmedWriteRequest | (SN only) |
| InformationReportRequest | EventNotification |

### Address Format

| Mode | Address Size | Format |
|------|--------------|--------|
| LN | 6 bytes | OBIS code |
| SN | 2 bytes | base_name (uint16) |

---

## Implementation Order

1. Association module (state, context, events)
2. AARQ/AARE encapsulation
3. OPEN/RELEASE service primitives
4. Encrypted PDU enumeration and methods
5. SN addressing PDU
6. Integration testing

---

## Files to Modify/Create

**New:**
- `dlms-application/src/association/mod.rs`
- `dlms-application/src/association/state.rs`
- `dlms-application/src/association/context.rs`
- `dlms-application/src/association/events.rs`

**Modify:**
- `dlms-application/src/lib.rs`
- `dlms-application/src/pdu.rs`
- `dlms-application/src/addressing.rs`
- `dlms-asn1/src/iso_acse/pdu.rs`
