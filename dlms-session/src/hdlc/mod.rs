//! HDLC session layer module

pub mod frame;
pub mod address;
pub mod decoder;
pub mod dispatcher;
pub mod connection;
pub mod fcs;

pub use frame::{FrameType, HdlcFrame, FLAG, LLC_REQUEST};
pub use address::{HdlcAddress, HdlcAddressPair, reserved};
pub use decoder::HdlcMessageDecoder;
pub use dispatcher::{HdlcDispatcher, HdlcMessageQueue};
pub use connection::{HdlcConnection, HdlcParameters};
pub use fcs::FcsCalc;
