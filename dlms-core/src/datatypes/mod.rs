//! Data types used in DLMS/COSEM protocol

pub mod data_object;
pub mod bit_string;
pub mod compact_array;
pub mod cosem_date;
pub mod cosem_time;
pub mod cosem_date_time;

// Re-export types
pub use bit_string::BitString;
pub use compact_array::{CompactArray, TypeDesc as CompactArrayTypeDesc, Type as CompactArrayType, DescriptionArray};
pub use cosem_date::{CosemDate, CosemDateFormat, Field, Month};
pub use cosem_time::CosemTime;
pub use cosem_date_time::{CosemDateTime, ClockStatus};
pub use data_object::{DataObject, DataObjectType};

// Re-export types when implemented
// pub use data_object::{DataObject, DataObjectType};
// pub use compact_array::CompactArray;
