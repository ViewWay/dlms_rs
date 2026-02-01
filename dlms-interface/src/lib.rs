//! COSEM Interface Classes for DLMS/COSEM Protocol
//!
//! This crate provides COSEM (Companion Specification for Energy Metering) interface
//! class definitions and implementations as specified in the DLMS/COSEM standards.
//!
//! # Overview
//!
//! COSEM interface classes define the standard objects used in smart metering systems.
//! Each interface class has:
//!
//! - A **Class ID** that uniquely identifies the class
//! **Attributes** that represent the object's properties
//! - **Methods** that represent actions the object can perform
//! - An **OBIS code** that identifies specific instances (for Logical Name addressing)
//!
//! # Core Interface Classes
//!
//! ## Data (Class ID: 1)
//!
//! Generic data storage for arbitrary COSEM data:
//!
//! ```rust,ignore
//! use dlms_interface::Data;
//! use dlms_core::{DataObject, ObisCode};
//!
//! let data = Data::new(ObisCode::new(1, 1, 0, 0, 0, 255));
//! data.set_value(DataObject::from("Hello")).await?;
//! ```
//!
//! ## Register (Class ID: 3)
//!
//! Stores a single value with scaler and unit:
//!
//! ```rust,ignore
//! use dlms_interface::{Register, ScalerUnit};
//! use dlms_core::{DataObject, ObisCode};
//!
//! let register = Register::new(ObisCode::new(1, 1, 1, 8, 0, 255));
//! register.set_value(DataObject::new_unsigned32(12345)).await?;
//!
//! // Read with scaler/unit applied
//! let scaled = register.get_scaled_value().await?;
//! println!("Value: {} {}", scaled.value, scaled.unit);
//! ```
//!
//! ## Extended Register (Class ID: 4)
//!
//! Extended register with status information:
//!
//! ```rust,ignore
//! use dlms_interface::ExtendedRegister;
//! use dlms_core::ObisCode;
//!
//! let ext_register = ExtendedRegister::new(ObisCode::new(1, 1, 1, 8, 1, 255));
//! // Has additional status field
//! ```
//!
//! ## Clock (Class ID: 8)
//!
//! Real-time clock with timezone support:
//!
//! ```rust,ignore
//! use dlms_interface::Clock;
//! use dlms_core::ObisCode;
//!
//! let clock = Clock::new(ObisCode::new(1, 0, 0, 9, 0, 255));
//! clock.sync_to_system().await?;
//!
//! let now = clock.get_datetime().await?;
//! println!("Current time: {}", now);
//! ```
//!
//! ## Profile Generic (Class ID: 7)
//!
//! Stores time-series data (load profiles, event logs):
//!
//! ```rust,ignore
//! use dlms_interface::{ProfileGeneric, ProfileSortMethod};
//! use dlms_core::ObisCode;
//!
//! let profile = ProfileGeneric::new(ObisCode::new(1, 1, 99, 1, 0, 255));
//! profile.set_sort_method(ProfileSortMethod::FIFO).await?;
//!
//! // Add entries to the profile
//! // profile.add_entry(...).await?;
//!
//! // Read profile data
//! let entries = profile.get_entries(0, 10).await?;
//! ```
//!
//! # Association Objects
//!
//! ## Association Logical Name (Class ID: 15)
//!
//! Manages logical name associations between client and server:
//!
//! ```rust,ignore
//! use dlms_interface::AssociationLn;
//! use dlms_core::ObisCode;
//!
//! let assoc = AssociationLn::new(ObisCode::new(0, 0, 40, 0, 0, 255));
//! // Manages client ID, authentication, and access rights
//! ```
//!
//! ## Association Short Name (Class ID: 12)
//!
//! Manages short name associations:
//!
//! ```rust,ignore
//! use dlms_interface::AssociationSn;
//! use dlms_core::ObisCode;
//!
//! let assoc = AssociationSn::new(ObisCode::new(0, 0, 40, 0, 1, 255));
//! // Manages short name to object mappings
//! ```
//!
//! # Utility Objects
//!
//! ## Script Table (Class ID: 9)
//!
//! Stores and executes scripts:
//!
//! ```rust,ignore
//! use dlms_interface::ScriptTable;
//! use dlms_core::ObisCode;
//!
//! let scripts = ScriptTable::new(ObisCode::new(1, 0, 0, 10, 0, 255));
//! // Execute a script
//! let result = scripts.execute(1).await?;
//! ```
//!
//! ## Schedule (Class ID: 10)
//!
//! Time-based scheduling:
//!
//! ```rust,ignore
//! use dlms_interface::{Schedule, ScheduleEntry};
//! use dlms_core::ObisCode;
//!
//! let schedule = Schedule::new(ObisCode::new(1, 0, 0, 11, 0, 255));
//! schedule.add_entry(ScheduleEntry::new()).await?;
//! ```
//!
//! ## Special Days Table (Class ID: 11)
//!
//! Calendar special days (holidays):
//!
//! ```rust,ignore
//! use dlms_interface::{SpecialDaysTable, SpecialDayEntry};
//! use dlms_core::ObisCode;
//!
//! let table = SpecialDaysTable::new(ObisCode::new(1, 0, 0, 12, 0, 255));
//! table.add_special_day(SpecialDayEntry::new(1, 2024, 12, 25)).await?;
//! ```
//!
//! # Setup Objects
//!
//! ## IEC HDLC Setup (Class ID: 23)
//!
//! HDLC communication parameters:
//!
//! ```rust,ignore
//! use dlms_interface::IecHdlcSetup;
//! use dlms_core::ObisCode;
//!
//! let setup = IecHdlcSetup::new(ObisCode::new(0, 0, 23, 0, 0, 255));
//! setup.set_communication_speed(9600).await?;
//! ```
//!
//! ## Security Setup (Class ID: 64)
//!
//! Security-related configuration:
//!
//! ```rust,ignore
//! use dlms_interface::SecuritySetup;
//! use dlms_core::ObisCode;
//!
//! let security = SecuritySetup::new(ObisCode::new(0, 0, 64, 0, 0, 255));
//! // Configure encryption and authentication
//! ```
//!
//! # Control Objects
//!
//! ## Disconnect Control (Class ID: 70)
//!
//! Remote disconnect/connect control:
//!
//! ```rust,ignore
//! use dlms_interface::{DisconnectControl, OutputState};
//! use dlms_core::ObisCode;
//!
//! let dc = DisconnectControl::new(ObisCode::new(0, 0, 96, 11, 0, 255));
//! dc.set_output_state(OutputState::Connected).await?;
//! ```
//!
//! ## Limiter (Class ID: 71)
//!
//! Power limiting control:
//!
//! ```rust,ignore
//! use dlms_interface::Limiter;
//! use dlms_core::ObisCode;
//!
//! let limiter = Limiter::new(ObisCode::new(0, 0, 96, 12, 0, 255));
//! // Set power limits and control actions
//! ```
//!
//! # CosemObject Trait
//!
//! All interface classes implement the [`CosemObject`] trait for unified access:
//!
//! ```rust,ignore
//! use dlms_interface::CosemObject;
//! use async_trait::async_trait;
//!
//! async fn read_object(object: &dyn CosemObject) -> Result<DataObject, Error> {
//!     // Read attribute 2 (value)
//!     object.get_attribute(2, None).await
//! }
//! ```
//!
//! # Attribute and Method Handling
//!
//! The crate provides utilities for attribute and method management:
//!
//! ## Attribute Registry
//!
//! ```rust,ignore
//! use dlms_interface::{AttributeRegistry, AttributeMetadata};
//!
//! let registry = AttributeRegistry::new();
//! registry.register(AttributeMetadata::new(2, "value", AccessMode::Read))?;
//! ```
//!
//! ## Method Registry
//!
//! ```rust,ignore
//! use dlms_interface::{MethodRegistry, MethodMetadata};
//!
//! let registry = MethodRegistry::new();
//! registry.register(MethodMetadata::new(1, "execute"))?;
//! ```
//!
//! # Macro System
//!
//! The crate provides macros for defining interface classes:
//!
//! ```rust,ignore
//! use dlms_interface::cosem_class;
//!
//! cosem_class! {
//!     /// My Custom Interface Class
//!     pub struct MyClass {
//!         // Attributes defined here
//!     }
//! }
//! ```
//!
//! # Scaler and Unit
//!
//! The [`ScalerUnit`] type represents the scaling factor and physical unit:
//!
//! ```rust,ignore
//! use dlms_interface::{ScalerUnit, units};
//!
//! // Create for kilowatt-hours (kWh)
//! let scaler = ScalerUnit::new(-3, units::Energy::KilowattHour);
//!
//! // Apply to raw value
//! let scaled = scaler.apply(12345); // 12.345 kWh
//! ```
//!
//! # Implementation Status
//!
//! ## Interface Classes (50+ implemented)
//! - [x] Data (Class ID: 1)
//! - [x] Data Store (Class ID: 2)
//! - [x] Register (Class ID: 3)
//! - [x] Extended Register (Class ID: 4)
//! - [x] Demand Register (Class ID: 5)
//! - [x] Register Activation (Class ID: 6)
//! - [x] Profile Generic (Class ID: 7)
//! - [x] Clock (Class ID: 8)
//! - [x] Script Table (Class ID: 9)
//! - [x] Schedule (Class ID: 10)
//! - [x] Special Days Table (Class ID: 11)
//! - [x] Association SN (Class ID: 12)
//! - [x] Association LN (Class ID: 15)
//! - [x] SAP Assignment (Class ID: 17)
//! - [x] Image Transfer (Class ID: 18)
//! - [x] IEC Local Port Setup (Class ID: 19)
//! - [x] Activity Calendar (Class ID: 20)
//! - [x] Register Monitor (Class ID: 21)
//! - [x] Single Action Schedule (Class ID: 22)
//! - [x] IEC HDLC Setup (Class ID: 23)
//! - [x] IEC Twisted Pair Setup (Class ID: 24)
//! - [x] MBus Slave Port Setup (Class ID: 25)
//! - [x] Generic Setup (Class ID: 26)
//! - [x] GPRS Setup (Class ID: 63)
//! - [x] Security Setup (Class ID: 64)
//! - [x] Auto Connect (Class ID: 66)
//! - [x] Transfer/Account (Class ID: 67)
//! - [x] Status Mapping (Class ID: 68)
//! - [x] TCP/UDP Setup (Class ID: 69)
//! - [x] Disconnect Control (Class ID: 70)
//! - [x] Limiter (Class ID: 71)
//! - [x] IP4 Setup (Class ID: 72) - referenced as Ip4Setup
//! - [x] Push Setup (Class ID: 40)
//! - [x] Relief Register (Class ID: 87)
//! - [x] Firmware Controller (Class ID: 83)
//! - [x] Unit (Class ID: 3) - unit registry
//! - [x] Octet String (Class ID: 89)
//! - [x] String (Class ID: 90)
//! - [x] Boolean Array (Class ID: 91)
//! - [x] Compact Data (Class ID: 92)
//! - [x] Login (Class ID: 93)
//! - [x] Action Schedule (Class ID: 95)
//! - [x] Parameter Monitor (Class ID: 96)
//! - [x] Value Display (Class ID: 100)
//! - [x] Key Table (Class ID: 101)
//! - [x] Sensor (Class ID: 102)
//! - And many more...
//!
//! ## Attribute Handling
//! - [x] Attribute accessor implementation
//! - [x] Attribute value validation
//! - [x] Attribute access control
//! - [x] Attribute change notifications
//!
//! ## Method Handling
//! - [x] Method invocation implementation
//! - [x] Method parameter validation
//! - [x] Method return value handling
//!
//! ## Macro System
//! - [x] Interface class definition macro
//! - [x] Attribute definition macros
//! - [x] Method definition macros
//!
//! # Module Structure
//!
//! - [`attribute`] - Attribute handling traits and implementations
//! - [`method`] - Method handling traits and implementations
//! - [`macros`] - Macro system for interface classes
//! - [`data`] - Data interface class
//! - [`register`] - Register interface class
//! - [`clock`] - Clock interface class
//! - [`profile_generic`] - Profile generic interface class
//! - [`scaler_unit`] - Scaler and unit handling
//! - And 40+ more interface class modules
//!
//! # References
//!
//! - DLMS Green Book: COSEM Interface Classes
//! - DLMS Blue Book: DLMS/COSEM Architecture and Protocols
//! - IEC 62056-62: COSEM data model

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
pub mod extended_register_scaler;
pub mod led_display;
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
pub use register::{Register, RegisterChangeCallback};
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

// Attribute and method handling exports
pub use attribute::{
    AttributeAccess, AttributeMetadata, AttributeRegistry, AttributeValidator,
    AttributeAccessor, WithAttributes, MetadataValidator,
};
pub use method::{
    MethodResult, MethodMetadata, MethodRegistry, MethodInvoker,
    MethodHandler, MethodHandlerRegistry, MethodParameterValidator,
    MetadataMethodValidator, WithMethods, DataObjectType,
};

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
