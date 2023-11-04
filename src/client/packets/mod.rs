#![allow(dead_code)]

pub mod a2s_info;
pub mod a2s_info_reply;
pub mod s2c_challenge;

use std::fmt::Debug;

use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const SOURCE_PACKET_HEADER: i32 = -1;
pub const SOURCE_SIMPLE_PACKET_MAX_SIZE: usize = 1400;

pub type SourceChallenge = i32;

#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum QueryHeader {
    S2CChallenge = 0x41,
    A2SServerQueryGetChallenge = 0x57,
    A2SPlayer = 0x55,
    A2SPlayerReply = 0x44,
    A2SRules = 0x56,
    A2SRulesReply = 0x45,
    A2SInfo = 0x54,
    A2SInfoReply = 0x49,
    A2APing = 0x69,
    A2APingReply = 0x6A,
    GSInfo = 0x6D,
    GSInfoReply = 0x6E,
}

pub trait SourceQueryResponse: for<'a> TryFrom<&'a [u8]> + Into<Vec<u8>> + Sized + Debug + Clone {
    fn packet_header() -> QueryHeader;

    const SIZE: usize = std::mem::size_of::<Self>();
}

pub trait SourceQueryRequest: for<'a> TryFrom<&'a [u8]> + Into<Vec<u8>> + Debug + Clone {
    const SIZE: usize = std::mem::size_of::<Self>();

    fn new() -> Self;
    fn set_challenge(&mut self, challenge: SourceChallenge);
}
