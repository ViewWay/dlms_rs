//! Activity Calendar interface class (Class ID: 20)
//!
//! The Activity Calendar interface class manages seasonal calendar definitions
//! with start/end dates and associated day profiles. It is used for defining
//! different rate seasons (e.g., summer/winter rates) in smart metering.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: calendar_name_active - Active calendar name
//! - Attribute 3: season_profile_active - Active season profile
//! - Attribute 4: week_profile_table_active - Active week profile table
//! - Attribute 5: day_profile_table_active - Active day profile table
//! - Attribute 6: calendar_name_passive - Passive calendar name
//! - Attribute 7: season_profile_passive - Passive season profile
//! - Attribute 8: week_profile_table_passive - Passive week profile table
//! - Attribute 9: day_profile_table_passive - Passive day profile table
//!
//! # Methods
//!
//! - Method 1: activate_passive_calendar() - Activate the passive calendar

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDate, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;
use super::special_days_table::DayId;

/// Season Profile - defines a season with its start date and associated day profile
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeasonProfile {
    /// Season start date (month and day, year is not used)
    pub start_date: CosemDate,
    /// Month (1-12)
    pub month: u8,
    /// Day of month (1-31)
    pub day: u8,
    /// Day profile selector (which day profile to use for this season)
    pub day_id: DayId,
}

impl SeasonProfile {
    /// Create a new season profile
    pub fn new(start_date: CosemDate, month: u8, day: u8, day_id: DayId) -> Self {
        Self {
            start_date,
            month,
            day,
            day_id,
        }
    }

    /// Create from data object (structure)
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 4 => {
                let start_date = match &arr[0] {
                    DataObject::OctetString(bytes) if !bytes.is_empty() => {
                        CosemDate::decode(bytes.as_slice())?
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected OctetString for start_date".to_string(),
                        ))
                    }
                };
                let month = match &arr[1] {
                    DataObject::Unsigned8(m) => *m,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for month".to_string(),
                        ))
                    }
                };
                let day = match &arr[2] {
                    DataObject::Unsigned8(d) => *d,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for day".to_string(),
                        ))
                    }
                };
                let day_id = match &arr[3] {
                    DataObject::Enumerate(id) => DayId::from_u8(*id),
                    DataObject::Unsigned8(id) => DayId::from_u8(*id),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate for day_id".to_string(),
                        ))
                    }
                };
                Ok(Self {
                    start_date,
                    month,
                    day,
                    day_id,
                })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for SeasonProfile".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::OctetString(self.start_date.encode()),
            DataObject::Unsigned8(self.month),
            DataObject::Unsigned8(self.day),
            DataObject::Enumerate(self.day_id.to_u8()),
        ])
    }

    /// Validate the season profile
    pub fn validate(&self) -> DlmsResult<()> {
        if self.month < 1 || self.month > 12 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid month: {}, must be 1-12",
                self.month
            )));
        }
        if self.day < 1 || self.day > 31 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid day: {}, must be 1-31",
                self.day
            )));
        }
        Ok(())
    }
}

/// Week Profile - defines which day profile to use for each day of the week
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekProfile {
    /// Week profile identifier
    pub week_id: u8,
    /// Day profile ID for Monday
    pub monday: u8,
    /// Day profile ID for Tuesday
    pub tuesday: u8,
    /// Day profile ID for Wednesday
    pub wednesday: u8,
    /// Day profile ID for Thursday
    pub thursday: u8,
    /// Day profile ID for Friday
    pub friday: u8,
    /// Day profile ID for Saturday
    pub saturday: u8,
    /// Day profile ID for Sunday
    pub sunday: u8,
}

impl WeekProfile {
    /// Create a new week profile
    pub fn new(week_id: u8, monday: u8, tuesday: u8, wednesday: u8, thursday: u8,
               friday: u8, saturday: u8, sunday: u8) -> Self {
        Self {
            week_id,
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
        }
    }

    /// Create from data object (structure)
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 8 => {
                let week_id = match &arr[0] {
                    DataObject::Unsigned8(id) => *id,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for week_id".to_string(),
                        ))
                    }
                };
                let get_day = |idx: usize| -> DlmsResult<u8> {
                    match &arr[idx] {
                        DataObject::Unsigned8(d) => Ok(*d),
                        _ => Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for day".to_string(),
                        ))
                    }
                };
                Ok(Self {
                    week_id,
                    monday: get_day(1)?,
                    tuesday: get_day(2)?,
                    wednesday: get_day(3)?,
                    thursday: get_day(4)?,
                    friday: get_day(5)?,
                    saturday: get_day(6)?,
                    sunday: get_day(7)?,
                })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for WeekProfile".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::Unsigned8(self.week_id),
            DataObject::Unsigned8(self.monday),
            DataObject::Unsigned8(self.tuesday),
            DataObject::Unsigned8(self.wednesday),
            DataObject::Unsigned8(self.thursday),
            DataObject::Unsigned8(self.friday),
            DataObject::Unsigned8(self.saturday),
            DataObject::Unsigned8(self.sunday),
        ])
    }

    /// Get the day profile ID for a given weekday (0=Monday, 6=Sunday)
    pub fn get_day_profile(&self, weekday: u8) -> Option<u8> {
        match weekday {
            0 => Some(self.monday),
            1 => Some(self.tuesday),
            2 => Some(self.wednesday),
            3 => Some(self.thursday),
            4 => Some(self.friday),
            5 => Some(self.saturday),
            6 => Some(self.sunday),
            _ => None,
        }
    }
}

/// Day Profile - defines a script to execute for a specific day
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayProfile {
    /// Day profile identifier
    pub day_id: u8,
    /// Script ID to execute
    pub script_id: u8,
}

impl DayProfile {
    /// Create a new day profile
    pub fn new(day_id: u8, script_id: u8) -> Self {
        Self { day_id, script_id }
    }

    /// Create from data object (structure)
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 2 => {
                let day_id = match &arr[0] {
                    DataObject::Unsigned8(id) => *id,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for day_id".to_string(),
                        ))
                    }
                };
                let script_id = match &arr[1] {
                    DataObject::Unsigned8(id) => *id,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for script_id".to_string(),
                        ))
                    }
                };
                Ok(Self { day_id, script_id })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for DayProfile".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::Unsigned8(self.day_id),
            DataObject::Unsigned8(self.script_id),
        ])
    }
}

/// Calendar state containing all profiles for active or passive calendar
#[derive(Debug, Clone)]
pub struct CalendarState {
    /// Calendar name
    pub calendar_name: String,
    /// Season profiles
    pub season_profiles: Vec<SeasonProfile>,
    /// Week profiles
    pub week_profiles: Vec<WeekProfile>,
    /// Day profiles
    pub day_profiles: Vec<DayProfile>,
}

impl CalendarState {
    /// Create a new calendar state
    pub fn new(calendar_name: String) -> Self {
        Self {
            calendar_name,
            season_profiles: Vec::new(),
            week_profiles: Vec::new(),
            day_profiles: Vec::new(),
        }
    }

    /// Create a default calendar state
    pub fn default_state() -> Self {
        Self::new("Default".to_string())
    }

    /// Check if the calendar state is empty
    pub fn is_empty(&self) -> bool {
        self.season_profiles.is_empty()
            && self.week_profiles.is_empty()
            && self.day_profiles.is_empty()
    }
}

/// Activity Calendar interface class (Class ID: 20)
///
/// Default OBIS: 0-0:13.0.0.255
///
/// This class manages seasonal calendar definitions for scheduling purposes.
/// It maintains both active and passive calendar states, allowing for
/// seamless transitions between different calendar configurations.
#[derive(Debug, Clone)]
pub struct ActivityCalendar {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Active calendar state
    active_state: Arc<RwLock<CalendarState>>,

    /// Passive calendar state (used for pending calendar changes)
    passive_state: Arc<RwLock<CalendarState>>,
}

impl ActivityCalendar {
    /// Class ID for Activity Calendar
    pub const CLASS_ID: u16 = 20;

    /// Default OBIS code for Activity Calendar (0-0:13.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 13, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_CALENDAR_NAME_ACTIVE: u8 = 2;
    pub const ATTR_SEASON_PROFILE_ACTIVE: u8 = 3;
    pub const ATTR_WEEK_PROFILE_TABLE_ACTIVE: u8 = 4;
    pub const ATTR_DAY_PROFILE_TABLE_ACTIVE: u8 = 5;
    pub const ATTR_CALENDAR_NAME_PASSIVE: u8 = 6;
    pub const ATTR_SEASON_PROFILE_PASSIVE: u8 = 7;
    pub const ATTR_WEEK_PROFILE_TABLE_PASSIVE: u8 = 8;
    pub const ATTR_DAY_PROFILE_TABLE_PASSIVE: u8 = 9;

    /// Method IDs
    pub const METHOD_ACTIVATE_PASSIVE_CALENDAR: u8 = 1;

    /// Create a new Activity Calendar object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `active_state` - Initial active calendar state
    /// * `passive_state` - Initial passive calendar state
    pub fn new(logical_name: ObisCode, active_state: CalendarState, passive_state: CalendarState) -> Self {
        Self {
            logical_name,
            active_state: Arc::new(RwLock::new(active_state)),
            passive_state: Arc::new(RwLock::new(passive_state)),
        }
    }

    /// Create with default OBIS code and empty states
    pub fn with_default_obis() -> Self {
        Self::new(
            Self::default_obis(),
            CalendarState::default_state(),
            CalendarState::default_state(),
        )
    }

    /// Get the active calendar name
    pub async fn active_calendar_name(&self) -> String {
        self.active_state.read().await.calendar_name.clone()
    }

    /// Set the active calendar name
    pub async fn set_active_calendar_name(&self, name: String) {
        self.active_state.write().await.calendar_name = name;
    }

    /// Get the passive calendar name
    pub async fn passive_calendar_name(&self) -> String {
        self.passive_state.read().await.calendar_name.clone()
    }

    /// Set the passive calendar name
    pub async fn set_passive_calendar_name(&self, name: String) {
        self.passive_state.write().await.calendar_name = name;
    }

    /// Get all active season profiles
    pub async fn active_season_profiles(&self) -> Vec<SeasonProfile> {
        self.active_state.read().await.season_profiles.clone()
    }

    /// Add a season profile to the active calendar
    pub async fn add_active_season_profile(&self, profile: SeasonProfile) -> DlmsResult<()> {
        profile.validate()?;
        let mut state = self.active_state.write().await;
        state.season_profiles.push(profile);
        Ok(())
    }

    /// Clear all active season profiles
    pub async fn clear_active_season_profiles(&self) {
        self.active_state.write().await.season_profiles.clear();
    }

    /// Get all passive season profiles
    pub async fn passive_season_profiles(&self) -> Vec<SeasonProfile> {
        self.passive_state.read().await.season_profiles.clone()
    }

    /// Add a season profile to the passive calendar
    pub async fn add_passive_season_profile(&self, profile: SeasonProfile) -> DlmsResult<()> {
        profile.validate()?;
        let mut state = self.passive_state.write().await;
        state.season_profiles.push(profile);
        Ok(())
    }

    /// Get all active week profiles
    pub async fn active_week_profiles(&self) -> Vec<WeekProfile> {
        self.active_state.read().await.week_profiles.clone()
    }

    /// Add a week profile to the active calendar
    pub async fn add_active_week_profile(&self, profile: WeekProfile) {
        let mut state = self.active_state.write().await;
        state.week_profiles.push(profile);
    }

    /// Get all passive week profiles
    pub async fn passive_week_profiles(&self) -> Vec<WeekProfile> {
        self.passive_state.read().await.week_profiles.clone()
    }

    /// Add a week profile to the passive calendar
    pub async fn add_passive_week_profile(&self, profile: WeekProfile) {
        let mut state = self.passive_state.write().await;
        state.week_profiles.push(profile);
    }

    /// Get all active day profiles
    pub async fn active_day_profiles(&self) -> Vec<DayProfile> {
        self.active_state.read().await.day_profiles.clone()
    }

    /// Add a day profile to the active calendar
    pub async fn add_active_day_profile(&self, profile: DayProfile) {
        let mut state = self.active_state.write().await;
        state.day_profiles.push(profile);
    }

    /// Get all passive day profiles
    pub async fn passive_day_profiles(&self) -> Vec<DayProfile> {
        self.passive_state.read().await.day_profiles.clone()
    }

    /// Add a day profile to the passive calendar
    pub async fn add_passive_day_profile(&self, profile: DayProfile) {
        let mut state = self.passive_state.write().await;
        state.day_profiles.push(profile);
    }

    /// Activate the passive calendar (copies passive state to active state)
    ///
    /// This corresponds to Method 1
    pub async fn activate_passive_calendar(&self) -> DlmsResult<()> {
        let passive = self.passive_state.read().await.clone();
        let mut active = self.active_state.write().await;
        *active = passive;
        Ok(())
    }

    /// Encode season profiles as a DataObject
    fn encode_season_profiles(profiles: &[SeasonProfile]) -> DataObject {
        let data: Vec<DataObject> = profiles.iter()
            .map(|p| p.to_data_object())
            .collect();
        DataObject::Array(data)
    }

    /// Encode week profiles as a DataObject
    fn encode_week_profiles(profiles: &[WeekProfile]) -> DataObject {
        let data: Vec<DataObject> = profiles.iter()
            .map(|p| p.to_data_object())
            .collect();
        DataObject::Array(data)
    }

    /// Encode day profiles as a DataObject
    fn encode_day_profiles(profiles: &[DayProfile]) -> DataObject {
        let data: Vec<DataObject> = profiles.iter()
            .map(|p| p.to_data_object())
            .collect();
        DataObject::Array(data)
    }
}

#[async_trait]
impl CosemObject for ActivityCalendar {
    fn class_id(&self) -> u16 {
        Self::CLASS_ID
    }

    fn obis_code(&self) -> ObisCode {
        self.logical_name
    }

    async fn get_attribute(
        &self,
        attribute_id: u8,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<DataObject> {
        match attribute_id {
            Self::ATTR_LOGICAL_NAME => {
                Ok(DataObject::OctetString(self.logical_name.to_bytes().to_vec()))
            }
            Self::ATTR_CALENDAR_NAME_ACTIVE => {
                let name = self.active_calendar_name().await;
                Ok(DataObject::OctetString(name.into_bytes()))
            }
            Self::ATTR_SEASON_PROFILE_ACTIVE => {
                let profiles = self.active_season_profiles().await;
                Ok(Self::encode_season_profiles(&profiles))
            }
            Self::ATTR_WEEK_PROFILE_TABLE_ACTIVE => {
                let profiles = self.active_week_profiles().await;
                Ok(Self::encode_week_profiles(&profiles))
            }
            Self::ATTR_DAY_PROFILE_TABLE_ACTIVE => {
                let profiles = self.active_day_profiles().await;
                Ok(Self::encode_day_profiles(&profiles))
            }
            Self::ATTR_CALENDAR_NAME_PASSIVE => {
                let name = self.passive_calendar_name().await;
                Ok(DataObject::OctetString(name.into_bytes()))
            }
            Self::ATTR_SEASON_PROFILE_PASSIVE => {
                let profiles = self.passive_season_profiles().await;
                Ok(Self::encode_season_profiles(&profiles))
            }
            Self::ATTR_WEEK_PROFILE_TABLE_PASSIVE => {
                let profiles = self.passive_week_profiles().await;
                Ok(Self::encode_week_profiles(&profiles))
            }
            Self::ATTR_DAY_PROFILE_TABLE_PASSIVE => {
                let profiles = self.passive_day_profiles().await;
                Ok(Self::encode_day_profiles(&profiles))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Activity Calendar has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn set_attribute(
        &self,
        attribute_id: u8,
        value: DataObject,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<()> {
        match attribute_id {
            Self::ATTR_LOGICAL_NAME => {
                Err(DlmsError::AccessDenied(
                    "Attribute 1 (logical_name) is read-only".to_string(),
                ))
            }
            Self::ATTR_CALENDAR_NAME_ACTIVE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let name = String::from_utf8_lossy(&bytes).to_string();
                        self.set_active_calendar_name(name).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for calendar_name".to_string(),
                    )),
                }
            }
            Self::ATTR_SEASON_PROFILE_ACTIVE => {
                match value {
                    DataObject::Array(arr) => {
                        self.clear_active_season_profiles().await;
                        for item in arr {
                            let profile = SeasonProfile::from_data_object(&item)?;
                            self.add_active_season_profile(profile).await?;
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.clear_active_season_profiles().await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for season_profile".to_string(),
                    )),
                }
            }
            Self::ATTR_WEEK_PROFILE_TABLE_ACTIVE => {
                match value {
                    DataObject::Array(arr) => {
                        let mut state = self.active_state.write().await;
                        state.week_profiles.clear();
                        for item in arr {
                            let profile = WeekProfile::from_data_object(&item)?;
                            state.week_profiles.push(profile);
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.active_state.write().await.week_profiles.clear();
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for week_profile_table".to_string(),
                    )),
                }
            }
            Self::ATTR_DAY_PROFILE_TABLE_ACTIVE => {
                match value {
                    DataObject::Array(arr) => {
                        let mut state = self.active_state.write().await;
                        state.day_profiles.clear();
                        for item in arr {
                            let profile = DayProfile::from_data_object(&item)?;
                            state.day_profiles.push(profile);
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.active_state.write().await.day_profiles.clear();
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for day_profile_table".to_string(),
                    )),
                }
            }
            Self::ATTR_CALENDAR_NAME_PASSIVE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let name = String::from_utf8_lossy(&bytes).to_string();
                        self.set_passive_calendar_name(name).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for calendar_name".to_string(),
                    )),
                }
            }
            Self::ATTR_SEASON_PROFILE_PASSIVE => {
                match value {
                    DataObject::Array(arr) => {
                        let mut state = self.passive_state.write().await;
                        state.season_profiles.clear();
                        for item in arr {
                            let profile = SeasonProfile::from_data_object(&item)?;
                            state.season_profiles.push(profile);
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.passive_state.write().await.season_profiles.clear();
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for season_profile".to_string(),
                    )),
                }
            }
            Self::ATTR_WEEK_PROFILE_TABLE_PASSIVE => {
                match value {
                    DataObject::Array(arr) => {
                        let mut state = self.passive_state.write().await;
                        state.week_profiles.clear();
                        for item in arr {
                            let profile = WeekProfile::from_data_object(&item)?;
                            state.week_profiles.push(profile);
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.passive_state.write().await.week_profiles.clear();
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for week_profile_table".to_string(),
                    )),
                }
            }
            Self::ATTR_DAY_PROFILE_TABLE_PASSIVE => {
                match value {
                    DataObject::Array(arr) => {
                        let mut state = self.passive_state.write().await;
                        state.day_profiles.clear();
                        for item in arr {
                            let profile = DayProfile::from_data_object(&item)?;
                            state.day_profiles.push(profile);
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.passive_state.write().await.day_profiles.clear();
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for day_profile_table".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Activity Calendar has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn invoke_method(
        &self,
        method_id: u8,
        _parameters: Option<DataObject>,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>> {
        match method_id {
            Self::METHOD_ACTIVATE_PASSIVE_CALENDAR => {
                self.activate_passive_calendar().await?;
                Ok(None)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Activity Calendar has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_date(year: u16, month: u8, day: u8) -> CosemDate {
        CosemDate::new(year, month, day).unwrap()
    }

    #[tokio::test]
    async fn test_activity_calendar_class_id() {
        let calendar = ActivityCalendar::with_default_obis();
        assert_eq!(calendar.class_id(), 20);
    }

    #[tokio::test]
    async fn test_activity_calendar_obis_code() {
        let calendar = ActivityCalendar::with_default_obis();
        assert_eq!(calendar.obis_code(), ActivityCalendar::default_obis());
    }

    #[tokio::test]
    async fn test_activity_calendar_default_names() {
        let calendar = ActivityCalendar::with_default_obis();
        assert_eq!(calendar.active_calendar_name().await, "Default");
        assert_eq!(calendar.passive_calendar_name().await, "Default");
    }

    #[tokio::test]
    async fn test_season_profile_new() {
        let date = create_test_date(2024, 6, 15);
        let profile = SeasonProfile::new(date.clone(), 6, 15, DayId::NormalWorkingDay);
        assert_eq!(profile.month, 6);
        assert_eq!(profile.day, 15);
        assert_eq!(profile.day_id, DayId::NormalWorkingDay);
    }

    #[tokio::test]
    async fn test_season_profile_validate() {
        let date = create_test_date(2024, 6, 15);
        let valid_profile = SeasonProfile::new(date.clone(), 6, 15, DayId::NormalWorkingDay);
        assert!(valid_profile.validate().is_ok());

        let invalid_month = SeasonProfile::new(date.clone(), 13, 15, DayId::NormalWorkingDay);
        assert!(invalid_month.validate().is_err());

        let invalid_day = SeasonProfile::new(date, 6, 32, DayId::NormalWorkingDay);
        assert!(invalid_day.validate().is_err());
    }

    #[tokio::test]
    async fn test_season_profile_to_data_object() {
        let date = create_test_date(2024, 6, 15);
        let profile = SeasonProfile::new(date, 6, 15, DayId::PublicHoliday);
        let data = profile.to_data_object();

        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 4);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_season_profile_from_data_object() {
        let date = create_test_date(2024, 6, 15);
        let date_bytes = date.encode();
        let data = DataObject::Array(vec![
            DataObject::OctetString(date_bytes),
            DataObject::Unsigned8(6),
            DataObject::Unsigned8(15),
            DataObject::Enumerate(2),
        ]);

        let profile = SeasonProfile::from_data_object(&data).unwrap();
        assert_eq!(profile.month, 6);
        assert_eq!(profile.day, 15);
        assert_eq!(profile.day_id, DayId::PublicHoliday);
    }

    #[tokio::test]
    async fn test_week_profile_new() {
        let profile = WeekProfile::new(1, 1, 2, 3, 4, 5, 6, 7);
        assert_eq!(profile.week_id, 1);
        assert_eq!(profile.monday, 1);
        assert_eq!(profile.sunday, 7);
    }

    #[tokio::test]
    async fn test_week_profile_get_day_profile() {
        let profile = WeekProfile::new(1, 1, 2, 3, 4, 5, 6, 7);
        assert_eq!(profile.get_day_profile(0), Some(1)); // Monday
        assert_eq!(profile.get_day_profile(6), Some(7)); // Sunday
        assert_eq!(profile.get_day_profile(3), Some(4)); // Thursday
        assert_eq!(profile.get_day_profile(99), None); // Invalid
    }

    #[tokio::test]
    async fn test_week_profile_to_data_object() {
        let profile = WeekProfile::new(1, 1, 2, 3, 4, 5, 6, 7);
        let data = profile.to_data_object();

        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 8);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_week_profile_from_data_object() {
        let data = DataObject::Array(vec![
            DataObject::Unsigned8(1),
            DataObject::Unsigned8(1),
            DataObject::Unsigned8(2),
            DataObject::Unsigned8(3),
            DataObject::Unsigned8(4),
            DataObject::Unsigned8(5),
            DataObject::Unsigned8(6),
            DataObject::Unsigned8(7),
        ]);

        let profile = WeekProfile::from_data_object(&data).unwrap();
        assert_eq!(profile.week_id, 1);
        assert_eq!(profile.monday, 1);
        assert_eq!(profile.sunday, 7);
    }

    #[tokio::test]
    async fn test_day_profile_new() {
        let profile = DayProfile::new(5, 10);
        assert_eq!(profile.day_id, 5);
        assert_eq!(profile.script_id, 10);
    }

    #[tokio::test]
    async fn test_day_profile_to_data_object() {
        let profile = DayProfile::new(5, 10);
        let data = profile.to_data_object();

        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_day_profile_from_data_object() {
        let data = DataObject::Array(vec![
            DataObject::Unsigned8(5),
            DataObject::Unsigned8(10),
        ]);

        let profile = DayProfile::from_data_object(&data).unwrap();
        assert_eq!(profile.day_id, 5);
        assert_eq!(profile.script_id, 10);
    }

    #[tokio::test]
    async fn test_calendar_state_new() {
        let state = CalendarState::new("TestCalendar".to_string());
        assert_eq!(state.calendar_name, "TestCalendar");
        assert!(state.is_empty());
    }

    #[tokio::test]
    async fn test_activity_calendar_set_active_name() {
        let calendar = ActivityCalendar::with_default_obis();
        calendar.set_active_calendar_name("SummerRates".to_string()).await;
        assert_eq!(calendar.active_calendar_name().await, "SummerRates");
    }

    #[tokio::test]
    async fn test_activity_calendar_set_passive_name() {
        let calendar = ActivityCalendar::with_default_obis();
        calendar.set_passive_calendar_name("WinterRates".to_string()).await;
        assert_eq!(calendar.passive_calendar_name().await, "WinterRates");
    }

    #[tokio::test]
    async fn test_activity_calendar_add_season_profile() {
        let calendar = ActivityCalendar::with_default_obis();
        let date = create_test_date(2024, 6, 15);
        let profile = SeasonProfile::new(date, 6, 15, DayId::NormalWorkingDay);

        calendar.add_active_season_profile(profile).await.unwrap();
        let profiles = calendar.active_season_profiles().await;
        assert_eq!(profiles.len(), 1);
    }

    #[tokio::test]
    async fn test_activity_calendar_add_week_profile() {
        let calendar = ActivityCalendar::with_default_obis();
        let profile = WeekProfile::new(1, 1, 2, 3, 4, 5, 6, 7);

        calendar.add_active_week_profile(profile).await;
        let profiles = calendar.active_week_profiles().await;
        assert_eq!(profiles.len(), 1);
    }

    #[tokio::test]
    async fn test_activity_calendar_add_day_profile() {
        let calendar = ActivityCalendar::with_default_obis();
        let profile = DayProfile::new(1, 5);

        calendar.add_active_day_profile(profile).await;
        let profiles = calendar.active_day_profiles().await;
        assert_eq!(profiles.len(), 1);
    }

    #[tokio::test]
    async fn test_activity_calendar_activate_passive() {
        let calendar = ActivityCalendar::with_default_obis();

        // Set up passive state
        calendar.set_passive_calendar_name("NewCalendar".to_string()).await;
        let date = create_test_date(2024, 6, 15);
        let profile = SeasonProfile::new(date, 6, 15, DayId::PublicHoliday);
        calendar.add_passive_season_profile(profile).await.unwrap();

        // Activate passive calendar
        calendar.activate_passive_calendar().await.unwrap();

        // Verify active state now has passive data
        assert_eq!(calendar.active_calendar_name().await, "NewCalendar");
        let profiles = calendar.active_season_profiles().await;
        assert_eq!(profiles.len(), 1);
    }

    #[tokio::test]
    async fn test_activity_calendar_get_logical_name() {
        let calendar = ActivityCalendar::with_default_obis();
        let result = calendar.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_activity_calendar_get_calendar_name_active() {
        let calendar = ActivityCalendar::with_default_obis();
        calendar.set_active_calendar_name("Summer".to_string()).await;

        let result = calendar.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes, b"Summer".to_vec());
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_activity_calendar_get_season_profile_active() {
        let calendar = ActivityCalendar::with_default_obis();
        let date = create_test_date(2024, 6, 15);
        let profile = SeasonProfile::new(date, 6, 15, DayId::NormalWorkingDay);
        calendar.add_active_season_profile(profile).await.unwrap();

        let result = calendar.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 1);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_activity_calendar_get_week_profile_active() {
        let calendar = ActivityCalendar::with_default_obis();
        let profile = WeekProfile::new(1, 1, 2, 3, 4, 5, 6, 7);
        calendar.add_active_week_profile(profile).await;

        let result = calendar.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 1);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_activity_calendar_get_day_profile_active() {
        let calendar = ActivityCalendar::with_default_obis();
        let profile = DayProfile::new(1, 5);
        calendar.add_active_day_profile(profile).await;

        let result = calendar.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 1);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_activity_calendar_set_calendar_name() {
        let calendar = ActivityCalendar::with_default_obis();
        calendar
            .set_attribute(2, DataObject::OctetString(b"Winter".to_vec()), None)
            .await
            .unwrap();

        assert_eq!(calendar.active_calendar_name().await, "Winter");
    }

    #[tokio::test]
    async fn test_activity_calendar_set_season_profile() {
        let calendar = ActivityCalendar::with_default_obis();
        let date = create_test_date(2024, 6, 15);
        let date_bytes = date.encode();
        let profile_data = DataObject::Array(vec![
            DataObject::OctetString(date_bytes),
            DataObject::Unsigned8(6),
            DataObject::Unsigned8(15),
            DataObject::Enumerate(0),
        ]);

        calendar
            .set_attribute(3, DataObject::Array(vec![profile_data]), None)
            .await
            .unwrap();

        let profiles = calendar.active_season_profiles().await;
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].month, 6);
    }

    #[tokio::test]
    async fn test_activity_calendar_set_null_clears() {
        let calendar = ActivityCalendar::with_default_obis();
        let date = create_test_date(2024, 6, 15);
        let profile = SeasonProfile::new(date, 6, 15, DayId::NormalWorkingDay);
        calendar.add_active_season_profile(profile).await.unwrap();

        calendar
            .set_attribute(3, DataObject::Null, None)
            .await
            .unwrap();

        assert!(calendar.active_season_profiles().await.is_empty());
    }

    #[tokio::test]
    async fn test_activity_calendar_read_only_logical_name() {
        let calendar = ActivityCalendar::with_default_obis();
        let result = calendar
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 13, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_activity_calendar_method_activate() {
        let calendar = ActivityCalendar::with_default_obis();
        calendar.set_passive_calendar_name("Activated".to_string()).await;

        let result = calendar.invoke_method(1, None, None).await.unwrap();
        assert!(result.is_none());

        assert_eq!(calendar.active_calendar_name().await, "Activated");
    }

    #[tokio::test]
    async fn test_activity_calendar_invalid_attribute() {
        let calendar = ActivityCalendar::with_default_obis();
        let result = calendar.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_activity_calendar_invalid_method() {
        let calendar = ActivityCalendar::with_default_obis();
        let result = calendar.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }
}
