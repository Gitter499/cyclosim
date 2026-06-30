mod crc;
mod encode;
mod types;
mod writer;

pub use crc::fit_crc16;
pub use encode::{encode_activity, FitEncodeError, FitRecordSample, FitRide, INDOOR_LAT_DEG, INDOOR_LON_DEG};
pub use types::{
    degrees_to_semicircles, distance_m_to_fit, duration_s_to_fit, fit_timestamp_to_unix,
    speed_mps_to_fit, unix_to_fit_timestamp, FitTimestamp, FIT_EPOCH_UNIX_OFFSET,
};
pub use writer::{BaseType, FieldDef, FitWriter};
