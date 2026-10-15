//! Packing/unpacking of Razer HID control reports.
//!
//! Report layout (90 bytes, matching the OpenRazer-documented struct used
//! across Razer Chroma peripherals):
//!
//! ```text
//! offset  size  field
//! 0       1     report id (always 0x00)
//! 1       1     status
//! 2       1     transaction id
//! 3       2     remaining packets (big-endian)
//! 5       1     protocol type (always 0x00)
//! 6       1     data size (length of the meaningful part of `arguments`)
//! 7       1     command class
//! 8       1     command id
//! 9       80    arguments (zero padded)
//! 89      1     crc (xor of bytes 2..=88 inclusive)
//! ```
//!
//! The CRC covers everything between the transaction id and the CRC byte
//! itself (i.e. not the report id or status byte).

use crate::constants::{ARGS_LENGTH, DEFAULT_TRANSACTION_ID, REPORT_LENGTH};

const CRC_START: usize = 2;
const CRC_END: usize = 88; // inclusive

fn crc(buf: &[u8; REPORT_LENGTH]) -> u8 {
    buf[CRC_START..=CRC_END].iter().fold(0u8, |acc, b| acc ^ b)
}

#[derive(Debug, Clone)]
pub struct RazerReport {
    pub command_class: u8,
    pub command_id: u8,
    pub arguments: Vec<u8>,
    pub status: u8,
    pub transaction_id: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("arguments too long: {0} > {ARGS_LENGTH}")]
    ArgumentsTooLong(usize),
    #[error("short report: {0} bytes, expected at least {REPORT_LENGTH}")]
    ShortReport(usize),
}

impl RazerReport {
    pub fn new(
        command_class: u8,
        command_id: u8,
        arguments: impl Into<Vec<u8>>,
    ) -> Result<Self, ProtocolError> {
        let arguments = arguments.into();
        if arguments.len() > ARGS_LENGTH {
            return Err(ProtocolError::ArgumentsTooLong(arguments.len()));
        }
        Ok(Self {
            command_class,
            command_id,
            arguments,
            status: 0x00,
            transaction_id: DEFAULT_TRANSACTION_ID,
        })
    }

    pub fn pack(&self) -> [u8; REPORT_LENGTH] {
        let mut buf = [0u8; REPORT_LENGTH];
        buf[0] = 0x00; // report id
        buf[1] = self.status;
        buf[2] = self.transaction_id;
        buf[3] = 0x00; // remaining packets high
        buf[4] = 0x00; // remaining packets low
        buf[5] = 0x00; // protocol type
        buf[6] = self.arguments.len() as u8;
        buf[7] = self.command_class;
        buf[8] = self.command_id;
        buf[9..9 + self.arguments.len()].copy_from_slice(&self.arguments);
        buf[89] = crc(&buf);
        buf
    }

    pub fn unpack(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < REPORT_LENGTH {
            return Err(ProtocolError::ShortReport(buf.len()));
        }
        let data_size = buf[6] as usize;
        let data_size = data_size.min(ARGS_LENGTH);
        Ok(Self {
            command_class: buf[7],
            command_id: buf[8],
            arguments: buf[9..9 + data_size].to_vec(),
            status: buf[1],
            transaction_id: buf[2],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_roundtrip_preserves_fields() {
        let report = RazerReport::new(0x04, 0x05, vec![0x00, 0x03, 0x20, 0x03, 0x20]).unwrap();
        let packed = report.pack();
        let unpacked = RazerReport::unpack(&packed).unwrap();

        assert_eq!(unpacked.command_class, 0x04);
        assert_eq!(unpacked.command_id, 0x05);
        assert_eq!(unpacked.arguments, vec![0x00, 0x03, 0x20, 0x03, 0x20]);
    }

    #[test]
    fn crc_is_xor_of_covered_range() {
        let report = RazerReport::new(0x00, 0x05, vec![0x01]).unwrap();
        let packed = report.pack();
        let expected = packed[CRC_START..=CRC_END]
            .iter()
            .fold(0u8, |acc, b| acc ^ b);
        assert_eq!(packed[89], expected);
    }

    #[test]
    fn rejects_oversized_arguments() {
        let too_long = vec![0u8; ARGS_LENGTH + 1];
        assert!(RazerReport::new(0x04, 0x05, too_long).is_err());
    }
}
