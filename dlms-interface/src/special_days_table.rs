//! Special Days Table interface class (Class ID: 11)
//!
//! The Special Days Table interface class stores special calendar days
//! such as holidays, working days, etc. with their associated day types.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: special_days_table - List of special day entries
//!
//! # Methods
//!
//! - Method 1: add_special_day(date, day_id) - Add a special day
//! - Method 2: remove_special_day(date) - Remove a special day
//! - Method 3: get_special_day(date) - Get the day ID for a date
//!
//! # Special Days Table (Class ID: 11)
//!
//! This class manages special calendar days that affect scheduling,
//! such as holidays, weekends with special schedules, etc.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDate, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Day ID for special days
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DayId {
    /// Normal working day
    NormalWorkingDay = 0,
    /// Non-working day (weekend)
    NonWorkingDay = 1,
    /// Public holiday
    PublicHoliday = 2,
    /// Additional non-working day
    AdditionalNonWorkingDay = 3,
    /// Special working day (e.g., make-up day)
    SpecialWorkingDay = 4,
}

impl DayId {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NormalWorkingDay,
            1 => Self::NonWorkingDay,
            2 => Self::PublicHoliday,
            3 => Self::AdditionalNonWorkingDay,
            4 => Self::SpecialWorkingDay,
            _ => Self::NormalWorkingDay,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is a working day
    pub fn is_working_day(self) -> bool {
        matches!(self, Self::NormalWorkingDay | Self::SpecialWorkingDay)
    }

    /// Check if this is a non-working day
    pub fn is_non_working_day(self) -> bool {
        !self.is_working_day()
    }
}

/// Special day entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialDayEntry {
    /// The date of this special day
    pub date: CosemDate,
    /// The day ID
    pub day_id: DayId,
}

impl SpecialDayEntry {
    /// Create a new special day entry
    pub fn new(date: CosemDate, day_id: DayId) -> Self {
        Self { date, day_id }
    }

    /// Create from data object (array)
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 2 => {
                let date = match &arr[0] {
                    DataObject::OctetString(bytes) if !bytes.is_empty() => {
                        CosemDate::decode(bytes.as_slice())?
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected OctetString for date".to_string(),
                        ))
                    }
                };
                let day_id = match &arr[1] {
                    DataObject::Enumerate(id) => DayId::from_u8(*id),
                    DataObject::Unsigned8(id) => DayId::from_u8(*id),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate for day_id".to_string(),
                        ))
                    }
                };
                Ok(Self { date, day_id })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for SpecialDayEntry".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::OctetString(self.date.encode()),
            DataObject::Enumerate(self.day_id.to_u8()),
        ])
    }
}

/// Special Days Table interface class (Class ID: 11)
///
/// Default OBIS: 0-0:11.0.0.255
///
/// This class manages special calendar days for scheduling purposes.
#[derive(Debug, Clone)]
pub struct SpecialDaysTable {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Special days table
    entries: Arc<RwLock<Vec<SpecialDayEntry>>>,
}

impl SpecialDaysTable {
    /// Class ID for Special Days Table
    pub const CLASS_ID: u16 = 11;

    /// Default OBIS code for Special Days Table (0-0:11.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 11, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_SPECIAL_DAYS_TABLE: u8 = 2;

    /// Method IDs
    pub const METHOD_ADD_SPECIAL_DAY: u8 = 1;
    pub const METHOD_REMOVE_SPECIAL_DAY: u8 = 2;
    pub const METHOD_GET_SPECIAL_DAY: u8 = 3;

    /// Create a new Special Days Table object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get all special day entries
    pub async fn entries(&self) -> Vec<SpecialDayEntry> {
        self.entries.read().await.clone()
    }

    /// Add a special day entry
    pub async fn add_entry(&self, entry: SpecialDayEntry) {
        let mut entries = self.entries.write().await;
        // Remove existing entry for the same date if any
        entries.retain(|e| e.date.encode() != entry.date.encode());
        entries.push(entry);
        // Keep sorted by date (using encoded bytes for comparison)
        entries.sort_by_key(|e| e.date.encode());
    }

    /// Remove a special day entry by date
    pub async fn remove_entry(&self, date: &CosemDate) -> bool {
        let mut entries = self.entries.write().await;
        let initial_len = entries.len();
        entries.retain(|e| &e.date != date);
        entries.len() < initial_len
    }

    /// Get the day ID for a specific date
    pub async fn get_day_id(&self, date: &CosemDate) -> Option<DayId> {
        let entries = self.entries.read().await;
        entries.iter().find(|e| &e.date == date).map(|e| e.day_id)
    }

    /// Check if a date is a working day
    pub async fn is_working_day(&self, date: &CosemDate) -> bool {
        match self.get_day_id(date).await {
            Some(day_id) => day_id.is_working_day(),
            None => true, // Default to working day if not specified
        }
    }

    /// Clear all entries
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }

    /// Get the number of entries
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Check if the table is empty
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }
}

#[async_trait]
impl CosemObject for SpecialDaysTable {
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
            Self::ATTR_SPECIAL_DAYS_TABLE => {
                let entries = self.entries().await;
                let data: Vec<DataObject> =
                    entries.iter().map(|e| e.to_data_object()).collect();
                Ok(DataObject::Array(data))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Special Days Table has no attribute {}",
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
            Self::ATTR_SPECIAL_DAYS_TABLE => {
                match value {
                    DataObject::Array(arr) => {
                        self.clear().await;
                        for item in arr {
                            let entry = SpecialDayEntry::from_data_object(&item)?;
                            self.add_entry(entry).await;
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.clear().await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for special_days_table".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Special Days Table has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn invoke_method(
        &self,
        method_id: u8,
        parameters: Option<DataObject>,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>> {
        match method_id {
            Self::METHOD_ADD_SPECIAL_DAY => {
                match parameters {
                    Some(DataObject::Array(arr)) if arr.len() >= 2 => {
                        let date = match &arr[0] {
                            DataObject::OctetString(bytes) if !bytes.is_empty() => {
                                CosemDate::decode(bytes.as_slice())?
                            }
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected OctetString for date".to_string(),
                                ))
                            }
                        };
                        let day_id = match &arr[1] {
                            DataObject::Enumerate(id) => DayId::from_u8(*id),
                            DataObject::Unsigned8(id) => DayId::from_u8(*id),
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Enumerate for day_id".to_string(),
                                ))
                            }
                        };
                        self.add_entry(SpecialDayEntry::new(date, day_id)).await;
                        Ok(None)
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Method 1 expects Array parameter with date and day_id".to_string(),
                    )),
                }
            }
            Self::METHOD_REMOVE_SPECIAL_DAY => {
                match parameters {
                    Some(DataObject::OctetString(bytes)) if !bytes.is_empty() => {
                        let date = CosemDate::decode(bytes.as_slice())?;
                        self.remove_entry(&date).await;
                        Ok(None)
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Method 2 expects OctetString parameter with date".to_string(),
                    )),
                }
            }
            Self::METHOD_GET_SPECIAL_DAY => {
                match parameters {
                    Some(DataObject::OctetString(bytes)) if !bytes.is_empty() => {
                        let date = CosemDate::decode(bytes.as_slice())?;
                        let day_id = self.get_day_id(&date).await.unwrap_or(DayId::NormalWorkingDay);
                        Ok(Some(DataObject::Enumerate(day_id.to_u8())))
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Method 3 expects OctetString parameter with date".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Special Days Table has no method {}",
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
    async fn test_special_days_table_class_id() {
        let table = SpecialDaysTable::with_default_obis();
        assert_eq!(table.class_id(), 11);
    }

    #[tokio::test]
    async fn test_special_days_table_obis_code() {
        let table = SpecialDaysTable::with_default_obis();
        assert_eq!(table.obis_code(), SpecialDaysTable::default_obis());
    }

    #[tokio::test]
    async fn test_special_days_table_initial_state() {
        let table = SpecialDaysTable::with_default_obis();
        assert!(table.is_empty().await);
        assert_eq!(table.len().await, 0);
    }

    #[tokio::test]
    async fn test_special_days_table_add_entry() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);
        let entry = SpecialDayEntry::new(date.clone(), DayId::PublicHoliday);

        table.add_entry(entry.clone()).await;

        assert_eq!(table.len().await, 1);
        let day_id = table.get_day_id(&date).await;
        assert_eq!(day_id, Some(DayId::PublicHoliday));
    }

    #[tokio::test]
    async fn test_special_days_table_remove_entry() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);
        let entry = SpecialDayEntry::new(date.clone(), DayId::PublicHoliday);

        table.add_entry(entry).await;
        assert_eq!(table.len().await, 1);

        let removed = table.remove_entry(&date).await;
        assert!(removed);
        assert!(table.is_empty().await);
    }

    #[tokio::test]
    async fn test_special_days_table_replace_entry() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);

        table.add_entry(SpecialDayEntry::new(date.clone(), DayId::PublicHoliday)).await;
        table.add_entry(SpecialDayEntry::new(date, DayId::SpecialWorkingDay)).await;

        // Should still have 1 entry (replaced, not added)
        assert_eq!(table.len().await, 1);
    }

    #[tokio::test]
    async fn test_special_days_table_clear() {
        let table = SpecialDaysTable::with_default_obis();
        let date1 = create_test_date(2024, 12, 25);
        let date2 = create_test_date(2024, 1, 1);

        table.add_entry(SpecialDayEntry::new(date1, DayId::PublicHoliday)).await;
        table.add_entry(SpecialDayEntry::new(date2, DayId::PublicHoliday)).await;

        assert_eq!(table.len().await, 2);
        table.clear().await;
        assert!(table.is_empty().await);
    }

    #[tokio::test]
    async fn test_special_days_table_is_working_day() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);

        // Default is working day
        assert!(table.is_working_day(&date).await);

        table.add_entry(SpecialDayEntry::new(date.clone(), DayId::PublicHoliday)).await;
        assert!(!table.is_working_day(&date).await);

        // Get the date back to use it again
        let date = create_test_date(2024, 12, 25);
        table.add_entry(SpecialDayEntry::new(date.clone(), DayId::SpecialWorkingDay)).await;
        assert!(table.is_working_day(&date).await);
    }

    #[tokio::test]
    async fn test_day_id_from_u8() {
        assert_eq!(DayId::from_u8(0), DayId::NormalWorkingDay);
        assert_eq!(DayId::from_u8(1), DayId::NonWorkingDay);
        assert_eq!(DayId::from_u8(2), DayId::PublicHoliday);
        assert_eq!(DayId::from_u8(3), DayId::AdditionalNonWorkingDay);
        assert_eq!(DayId::from_u8(4), DayId::SpecialWorkingDay);
        assert_eq!(DayId::from_u8(99), DayId::NormalWorkingDay);
    }

    #[tokio::test]
    async fn test_day_id_is_working_day() {
        assert!(DayId::NormalWorkingDay.is_working_day());
        assert!(!DayId::NonWorkingDay.is_working_day());
        assert!(!DayId::PublicHoliday.is_working_day());
        assert!(!DayId::AdditionalNonWorkingDay.is_working_day());
        assert!(DayId::SpecialWorkingDay.is_working_day());
    }

    #[tokio::test]
    async fn test_day_id_is_non_working_day() {
        assert!(!DayId::NormalWorkingDay.is_non_working_day());
        assert!(DayId::NonWorkingDay.is_non_working_day());
        assert!(DayId::PublicHoliday.is_non_working_day());
        assert!(DayId::AdditionalNonWorkingDay.is_non_working_day());
        assert!(!DayId::SpecialWorkingDay.is_non_working_day());
    }

    #[tokio::test]
    async fn test_special_day_entry_to_data_object() {
        let date = create_test_date(2024, 12, 25);
        let entry = SpecialDayEntry::new(date, DayId::PublicHoliday);
        let data = entry.to_data_object();

        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_special_day_entry_from_data_object() {
        let date = create_test_date(2024, 12, 25);
        let date_bytes = date.encode();
        let data = DataObject::Array(vec![
            DataObject::OctetString(date_bytes),
            DataObject::Enumerate(2),
        ]);

        let entry = SpecialDayEntry::from_data_object(&data).unwrap();
        assert_eq!(entry.day_id, DayId::PublicHoliday);
    }

    #[tokio::test]
    async fn test_special_days_table_get_logical_name() {
        let table = SpecialDaysTable::with_default_obis();
        let result = table.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_special_days_table_get_special_days_table() {
        let table = SpecialDaysTable::with_default_obis();
        let result = table.get_attribute(2, None).await.unwrap();

        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 0);
            }
            _ => panic!("Expected Array"),
        }

        let date = create_test_date(2024, 12, 25);
        table.add_entry(SpecialDayEntry::new(date, DayId::PublicHoliday)).await;

        let result = table.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 1);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_special_days_table_set_via_attribute() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);
        let date_bytes = date.encode();
        let entry_data = DataObject::Array(vec![
            DataObject::OctetString(date_bytes),
            DataObject::Enumerate(2),
        ]);

        table
            .set_attribute(2, DataObject::Array(vec![entry_data]), None)
            .await
            .unwrap();

        assert_eq!(table.len().await, 1);
    }

    #[tokio::test]
    async fn test_special_days_table_set_null_clears() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);
        table.add_entry(SpecialDayEntry::new(date, DayId::PublicHoliday)).await;

        table.set_attribute(2, DataObject::Null, None).await.unwrap();
        assert!(table.is_empty().await);
    }

    #[tokio::test]
    async fn test_special_days_table_read_only_logical_name() {
        let table = SpecialDaysTable::with_default_obis();
        let result = table
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 11, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_special_days_table_method_add() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);
        let date_bytes = date.encode();
        let params = DataObject::Array(vec![
            DataObject::OctetString(date_bytes),
            DataObject::Enumerate(2),
        ]);

        table.invoke_method(1, Some(params), None).await.unwrap();
        assert_eq!(table.len().await, 1);
        assert_eq!(table.get_day_id(&date).await, Some(DayId::PublicHoliday));
    }

    #[tokio::test]
    async fn test_special_days_table_method_remove() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);
        table.add_entry(SpecialDayEntry::new(date.clone(), DayId::PublicHoliday)).await;

        table
            .invoke_method(2, Some(DataObject::OctetString(date.encode())), None)
            .await
            .unwrap();

        assert!(table.is_empty().await);
    }

    #[tokio::test]
    async fn test_special_days_table_method_get() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);
        table.add_entry(SpecialDayEntry::new(date.clone(), DayId::PublicHoliday)).await;

        let result = table
            .invoke_method(3, Some(DataObject::OctetString(date.encode())), None)
            .await
            .unwrap();

        assert_eq!(result, Some(DataObject::Enumerate(2)));
    }

    #[tokio::test]
    async fn test_special_days_table_method_get_default() {
        let table = SpecialDaysTable::with_default_obis();
        let date = create_test_date(2024, 12, 25);

        let date_bytes = date.encode();
        let result = table
            .invoke_method(3, Some(DataObject::OctetString(date_bytes)), None)
            .await
            .unwrap();

        // Returns NormalWorkingDay (0) when not found
        assert_eq!(result, Some(DataObject::Enumerate(0)));
    }

    #[tokio::test]
    async fn test_special_days_table_invalid_attribute() {
        let table = SpecialDaysTable::with_default_obis();
        let result = table.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_special_days_table_invalid_method() {
        let table = SpecialDaysTable::with_default_obis();
        let result = table.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }
}
