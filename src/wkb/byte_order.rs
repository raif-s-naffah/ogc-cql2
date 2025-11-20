// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Representation of WKB blob data endianess indicator.
//!

/// How to interpret bytes representing signed and unsigned 32-bit integers and
/// 64-bit doubles.
#[derive(Debug)]
pub(crate) struct ByteOrder(bool);

impl ByteOrder {
    /// When TRUE, order is Little Endian, aka NDR (Network Detection and
    /// Response); i.e. least significant byte first. When FALSE it's Big
    /// Endian, aka XDR (eXtended Detection and Response); i.e. most
    /// significant byte first).
    pub(crate) fn is_le(&self) -> bool {
        self.0
    }
}

impl From<u8> for ByteOrder {
    fn from(value: u8) -> Self {
        Self(value == 1)
    }
}
