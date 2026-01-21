//! COSEM interface classes module for DLMS/COSEM protocol
//!
//! This crate provides COSEM interface class definitions and implementations.
//!
//! # TODO
//!
//! ## 接口类定义
//! - [x] Data 接口类（Class ID: 1） - 已实现
//! - [x] Register 接口类（Class ID: 3） - 已实现（包含ScalerUnit支持）
//! - [x] Extended Register 接口类（Class ID: 4） - 已实现
//! - [x] Demand Register 接口类（Class ID: 5） - 已实现
//! - [x] Register Activation 接口类（Class ID: 6） - 已实现
//! - [x] Profile Generic 接口类（Class ID: 7） - 已实现
//! - [x] Clock 接口类（Class ID: 8） - 已实现
//! - [x] Script Table 接口类（Class ID: 9） - 已实现
//! - [x] Schedule 接口类（Class ID: 10） - 已实现
//! - [x] Special Days Table 接口类（Class ID: 11） - 已实现
//! - [x] Association Short Name 接口类（Class ID: 12） - 已实现
//! - [x] Association Logical Name 接口类（Class ID: 15） - 已实现
//! - [x] SAP Assignment 接口类（Class ID: 17） - 已实现
//! - [x] Image Transfer 接口类（Class ID: 18） - 已实现
//! - [x] IEC Local Port Setup 接口类（Class ID: 19） - 已实现
//! - [x] Activity Calendar 接口类（Class ID: 20） - 已实现
//! - [x] Register Monitor 接口类（Class ID: 21） - 已实现
//! - [x] Single Action Schedule 接口类（Class ID: 22） - 已实现
//! - [x] IEC HDLC Setup 接口类（Class ID: 23） - 已实现
//! - [x] IEC  twisted pair setup 接口类（Class ID: 24） - 已实现
//! - [x] MBus Slave Port Setup 接口类（Class ID: 25） - 已实现
//! - [x] Security Setup 接口类（Class ID: 64） - 已实现
//! - [x] Disconnect Control 接口类（Class ID: 70） - 已实现
//! - [x] Limiter 接口类（Class ID: 71） - 已实现
//! - [x] Push Setup 接口类（Class ID: 40） - 已实现
//! - [x] Data Store 接口类（Class ID: 2） - 已实现
//! - [x] Generic Setup 接口类（Class ID: 26） - 已实现
//! - [x] Status Mapping 接口类（Class ID: 68） - 已实现
//! - [x] Relief Register 接口类（Class ID: 87） - 已实现
//! - [x] Unit 接口类（Class ID: 3） - 已实现
//! - [x] TCP UDP Setup 接口类（Class ID: 69） - 已实现
//! - [x] Firmware Controller 接口类（Class ID: 83） - 已实现
//! - [x] Octet String 接口类（Class ID: 89） - 已实现
//! - [x] String 接口类（Class ID: 90） - 已实现
//! - [x] Boolean Array 接口类（Class ID: 91） - 已实现
//! - [x] Compact Data 接口类（Class ID: 92） - 已实现
//! - [x] Login 接口类（Class ID: 93） - 已实现
//! - [x] Action Schedule 接口类（Class ID: 95） - 已实现
//! - [x] Parameter Monitor 接口类（Class ID: 96） - 已实现
//! - [x] GPRS Setup 接口类（Class ID: 63） - 已实现
//! - [x] Value Display 接口类（Class ID: 100） - 已实现
//! - [x] Key Table 接口类（Class ID: 101） - 已实现
//! - [x] Sensor 接口类（Class ID: 102） - 已实现
//! - [ ] 更多接口类...
//!
//! ## 属性处理
//! - [ ] 属性访问器实现
//! - [ ] 属性值验证
//! - [ ] 属性访问权限检查
//!
//! ## 方法处理
//! - [ ] 方法调用实现
//! - [ ] 方法参数验证
//! - [ ] 方法返回值处理
//!
//! ## 宏系统
//! - [ ] 接口类定义宏
//! - [ ] 属性定义宏
//! - [ ] 方法定义宏

use dlms_core::{DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::SelectiveAccessDescriptor;
use async_trait::async_trait;

pub mod attribute;
pub mod method;
pub mod macros;
pub mod data;
pub mod scaler_unit;
pub mod register;
pub mod register_activation;
pub mod special_days_table;
pub mod descriptor;
pub mod clock;
pub mod profile_generic;
pub mod extended_register;
pub mod demand_register;
pub mod script_table;
pub mod schedule;
pub mod iec_local_port_setup;
pub mod iec_hdlc_setup;
pub mod iec_twisted_pair_setup;
pub mod mbus_slave_port_setup;
pub mod disconnect_control;
pub mod limiter;
pub mod push_setup;
pub mod register_monitor;
pub mod activity_calendar;
pub mod single_action_schedule;
pub mod sap_assignment;
pub mod image_transfer;
pub mod association_ln;
pub mod association_sn;
pub mod security_setup;
pub mod ip4_setup;
pub mod auto_connect;
pub mod account;
pub mod credit;
pub mod charge;
pub mod token_gateway;
pub mod payment_meter;
pub mod sms_controller;
pub mod gsm_controller;
pub mod modem_configuration;
pub mod utility_meter;
pub mod alarm;
pub mod tariff;
pub mod led_display;
pub mod extended_register_scaler;
pub mod ip6_setup;
pub mod mac_address_setup;
pub mod modem_process;
pub mod generic_setup;
pub mod data_store;
pub mod status_mapping;
pub mod relief_register;
pub mod unit;
pub mod tcp_udp_setup;
pub mod firmware_controller;
pub mod octet_string;
pub mod string;
pub mod boolean_array;
pub mod compact_data;
pub mod login;
pub mod action_schedule;
pub mod parameter_monitor;
pub mod gprs_setup;
pub mod value_display;
pub mod key_table;
pub mod sensor;

pub use data::Data;
pub use scaler_unit::{ScalerUnit, units};
pub use register::Register;
pub use register_activation::RegisterActivation;
pub use special_days_table::{SpecialDaysTable, SpecialDayEntry, DayId};
pub use clock::Clock;
pub use profile_generic::{ProfileGeneric, GenericProfileEntry, ProfileSortMethod, ProfileBufferStatus};
pub use extended_register::ExtendedRegister;
pub use demand_register::DemandRegister;
pub use script_table::{ScriptTable, ScriptAction, ScriptDescriptor, ScriptExecutionResult};
pub use schedule::{Schedule, ScheduleEntry};
pub use iec_local_port_setup::{IecLocalPortSetup, Parity, PortMode, BaudRate};
pub use iec_hdlc_setup::{IecHdlcSetup, InformationLength};
pub use iec_twisted_pair_setup::{IecTwistedPairSetup, CommunicationMode, ProtocolSelect};
pub use mbus_slave_port_setup::{MBusSlavePortSetup, MBusParity};
pub use disconnect_control::{DisconnectControl, OutputState};
pub use limiter::{Limiter, LimiterAction};
pub use push_setup::{
    PushSetup, PushObjectDefinition, PushDestinationMethod, CommunicationWindow,
};
pub use register_monitor::{
    RegisterMonitor, MonitorThreshold, MonitorAction, MonitoredValueRef, ThresholdDirection,
};
pub use activity_calendar::{
    ActivityCalendar, SeasonProfile, WeekProfile, DayProfile, CalendarState,
};
pub use single_action_schedule::{
    SingleActionSchedule, ActionScheduleType, ExecutionResult,
};
pub use sap_assignment::{SapAssignment, SapAssignmentEntry, ShortName as SapShortName};
pub use image_transfer::{
    ImageTransfer, ImageTransferStatus, ImageInfo,
};
pub use descriptor::{
    CosemObjectDescriptor, AccessRight, AccessMode,
    AttributeDescriptor, MethodDescriptor, UserInfo,
    CaptureObjectDefinition, ProfileEntry, SortMethod,
    ObisCodeExt,
};
pub use association_ln::AssociationLn;
pub use association_sn::{AssociationSn, ShortName};
pub use security_setup::SecuritySetup;
pub use ip4_setup::{Ip4Setup, IpAddressMethod, Ipv4Addr};
pub use auto_connect::{AutoConnect, AutoConnectMode, ConnectionStatus, ConnectionTimeWindow};
pub use account::{Account, CreditStatus};
pub use credit::{Credit, CreditType, CreditStatusType};
pub use charge::{Charge, ChargeType};
pub use token_gateway::{TokenGateway, TokenStatus, TokenType};
pub use payment_meter::{PaymentMeter, PaymentMethod, PaymentStatus};
pub use sms_controller::{SmsController, SmsSendStatus};
pub use gsm_controller::{GsmController, GsmConnectionStatus, SignalStrength};
pub use modem_configuration::{ModemConfiguration, ModemBaudRate, ErrorControl};
pub use utility_meter::{UtilityMeter, UtilityMeterType, ReadingStatus};
pub use alarm::{Alarm, AlarmType, AlarmStatus};
pub use tariff::{Tariff, TariffType};
pub use extended_register_scaler::ExtendedRegisterScaler;
pub use led_display::{LedDisplay, DisplayMode};
pub use ip6_setup::{Ip6Setup, Ip6AddressMethod, Ipv6Addr};
pub use mac_address_setup::{MacAddressSetup, MacAddr};
pub use modem_process::{ModemProcess, ModemStatus, ModemConnectionStatus};
pub use generic_setup::GenericSetup;
pub use data_store::{DataStore, DataStoreType};
pub use status_mapping::{StatusMapping, StatusMappingEntry};
pub use relief_register::{ReliefRegister, ReliefStatus};
pub use unit::{Unit, UnitId};
pub use tcp_udp_setup::{TcpUdpSetup, ProtocolType};
pub use firmware_controller::{FirmwareController, FirmwareUpdateStatus};
pub use octet_string::OctetString;
pub use string::StringInterface;
pub use boolean_array::BooleanArray;
pub use compact_data::CompactData;
pub use login::{Login, AuthenticationStatus};
pub use action_schedule::ActionSchedule;
pub use parameter_monitor::ParameterMonitor;
pub use gprs_setup::{GprsSetup, QualityOfService};
pub use value_display::ValueDisplay;
pub use key_table::{KeyTable, KeyType};
pub use sensor::{Sensor, SensorStatus};

/// COSEM Object trait
///
/// This trait defines the interface that all COSEM objects must implement.
/// It provides methods for getting and setting attributes, and invoking methods.
///
/// # Why This Trait?
/// Using a trait allows:
/// - **Polymorphism**: Same code works with different object types
/// - **Extensibility**: Easy to add new object types
/// - **Testability**: Easy to mock objects for testing
#[async_trait]
pub trait CosemObject: Send + Sync {
    /// Get the class ID of this object
    fn class_id(&self) -> u16;

    /// Get the OBIS code (logical name) of this object
    fn obis_code(&self) -> ObisCode;

    /// Get an attribute value
    ///
    /// # Arguments
    /// * `attribute_id` - Attribute ID to read (1-255)
    /// * `selective_access` - Optional selective access descriptor
    ///
    /// # Returns
    /// The attribute value as a `DataObject`, or error if attribute doesn't exist
    async fn get_attribute(
        &self,
        attribute_id: u8,
        selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<DataObject>;

    /// Set an attribute value
    ///
    /// # Arguments
    /// * `attribute_id` - Attribute ID to write (1-255)
    /// * `value` - Value to write
    /// * `selective_access` - Optional selective access descriptor
    ///
    /// # Returns
    /// `Ok(())` if successful, error otherwise
    async fn set_attribute(
        &self,
        attribute_id: u8,
        value: DataObject,
        selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<()>;

    /// Invoke a method
    ///
    /// # Arguments
    /// * `method_id` - Method ID to invoke (1-255)
    /// * `parameters` - Method parameters (optional)
    /// * `selective_access` - Optional selective access descriptor
    ///
    /// # Returns
    /// The method return value as a `DataObject`, or error if method doesn't exist or fails
    async fn invoke_method(
        &self,
        method_id: u8,
        parameters: Option<DataObject>,
        selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>>;
}
