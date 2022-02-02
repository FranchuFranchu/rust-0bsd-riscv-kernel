use crate::MessageID;
use crate::ThreemaID;
use flat_bytes::flat_enum;
use flat_bytes::Flat;
use serde::de::Error;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::Deserializer;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};

flat_enum! {
    #[derive(Debug)]
    #[repr(u32)]
    pub enum Packet {
        EchoRequest(u64) = 0,
        EchoReply(u64) = 0x80,
        ClientToServer(Header) = 1,
        ServerToClient(Header) = 2,
        ServerAck(ThreemaID, MessageID) = 0x81,
        ClientAck(ThreemaID, MessageID) = 0x82,
        ConnectionEstablished = 0xd0,
        // Error
        DublicateConnection = 0xe0,
        Alert = 0xe1,
    }
}

pub type BallotID = [u8; 8];

flat_enum! {
    #[derive(Debug)]
    #[repr(u8)]
    pub enum Message {
        Text(Text) = 1,
        Image,
        Location = 0x10,
        Video = 0x13,
        Audio = 0x14,
        // Poll {
        BallotCreate {
            poll_id: BallotID,
            details: Ballot,
        } = 0x15,
        BallotVote {
        // PollUpdate {
            sender: ThreemaID,
            poll_id: BallotID,
            updates: BallotUpdates,
        } = 0x16,
        File(File) = 0x17,
        ContactSetPhoto = 0x18,
        ContactDeletePhoto = 0x19,
        ContactRequestPhoto = 0x1a,
        GroupText = 0x41,
        GroupLocation = 0x42,
        GroupImage = 0x43,
        GroupVideo = 0x44,
        GroupAudio = 0x45,
        GroupFile = 0x46,
        GroupCreate = 0x4a,
        GroupRename = 0x4b,
        GroupLeave = 0x4c,
        GroupAddMember = 0x4d,
        GroupRemoveMember = 0x4e,
        GroupDestroy = 0x4f,
        GroupSetPhoto = 0x50,
        GroupRequestSync = 0x51,
        GroupBallotCreate = 0x52,
        GroupBallotVote = 0x53,
        GroupDeletePhoto = 0x54,
        VoipCallOffer = 0x60,
        VoipCallAnswer = 0x61,
        VoipIceCandiates = 0x62,
        VoipCallHangup = 0x63,
        VoipCallRinging = 0x64,
        DeliveryReceipt(MessageStatus, MessageID) = 0x80,
        TypingNotification = 0x90,
    }
}

flat_enum! {
    #[derive(Debug)]
    #[repr(u8)]
    pub enum MessageStatus {
        Delivered = 1,
        Read,
        Approved,
        Disapproved,
    }
}

#[derive(Debug, Flat)]
pub struct Header {
    pub sender: ThreemaID,
    pub receiver: ThreemaID,
    pub msg_id: MessageID,
    pub timestamp: u32,
    pub flags: u32,
    pub nickname: [u8; 32],
    pub nonce: [u8; 24],
}

#[derive(Debug)]
pub struct Text {
    pub message: String,
}

impl Flat for Text {
    fn serialize(&self) -> Vec<u8> {
        self.message.as_bytes().to_owned()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let message = String::from_utf8(data.to_owned()).ok()?;
        Some((Self { message }, data.len()))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderingType {
    /// Display as default file message
    File = 0,
    /// Display as media file message (e.g. image or audio message)
    Media = 1,
    /// Display as sticker (images with transparency, rendered without bubble)
    Sticker = 2,
}

impl Serialize for RenderingType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

struct EnumVisitor;

impl<'de> Visitor<'de> for EnumVisitor {
    type Value = RenderingType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(formatter, "u8")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            0 => Ok(RenderingType::File),
            1 => Ok(RenderingType::Media),
            2 => Ok(RenderingType::Sticker),
            x => Err(Error::invalid_value(Unexpected::Unsigned(x), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for RenderingType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u8(EnumVisitor)
    }
}

impl Default for RenderingType {
    fn default() -> Self {
        RenderingType::File
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    #[serde(rename = "b")]
    blob_id: String,
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "m")]
    pub mime: String,
    #[serde(rename = "t")]
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail_blob_id: Option<String>,
    #[serde(rename = "p")]
    pub thumbnail_mime: String,
    #[serde(rename = "s")]
    pub size: u64,
    #[serde(rename = "d")]
    pub description: String,
    #[serde(rename = "j")]
    rendering_type: RenderingType,
    #[serde(rename = "k")]
    encryption_key: String,
    #[serde(flatten)]
    pub unknown: std::collections::HashMap<String, serde_json::Value>,
}

impl Flat for File {
    fn serialize(&self) -> Vec<u8> {
        to_vec(self).unwrap()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let res = from_slice(data).ok()?;
        Some((res, data.len()))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PollChoice {
    #[serde(rename = "i")]
    pub id: u32,
    #[serde(rename = "n")]
    pub text: String,
    #[serde(rename = "o")]
    pub order: u32,
    #[serde(rename = "r")]
    pub results: Vec<u32>,
    #[serde(flatten)]
    pub unknown: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[repr(u8)]
pub enum BallotState {
    Open = 0,
    Closed = 1,
}

#[derive(Debug, Deserialize, Serialize)]
#[repr(u8)]
pub enum BallotType {
    ResultOnClose = 0,
    Intermediate = 1,
}

#[derive(Debug, Deserialize, Serialize)]
#[repr(u8)]
pub enum AssessmentType {
    Single = 0,
    Multiple = 1,
}

#[derive(Debug, Deserialize, Serialize)]
#[repr(u8)]
pub enum ChoiceType {
    Text = 0,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ballot {
    #[serde(rename = "d")]
    pub description: String,
    #[serde(rename = "c")]
    pub choices: Vec<PollChoice>,
    #[serde(rename = "p")]
    pub participants: Vec<String>,
    #[serde(rename = "s")]
    pub state: BallotState,
    #[serde(rename = "a")]
    pub assessment_type: AssessmentType,
    #[serde(rename = "t")]
    pub ballot_type: BallotType,
    #[serde(rename = "o")]
    pub choice_type: ChoiceType,
    #[serde(flatten)]
    pub unknown: std::collections::HashMap<String, serde_json::Value>,
}

impl Flat for Ballot {
    fn serialize(&self) -> Vec<u8> {
        to_vec(self).unwrap()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let res = from_slice(data).ok()?;
        Some((res, data.len()))
    }
}

#[deprecated = "please use Ballot instead"]
pub type PollDetails = Ballot;

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BallotUpdates {
    updates: Vec<(u32, u32)>,
}

impl Flat for BallotUpdates {
    fn serialize(&self) -> Vec<u8> {
        to_vec(self).unwrap()
    }

    fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
        let res = from_slice(data).ok()?;
        Some((res, data.len()))
    }
}

#[deprecated = "please use BallotUpdates instead"]
pub type PollUpdate = BallotUpdates;
