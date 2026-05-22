//! Types for event streams defined in MSC4471.

use js_int::UInt;
use ruma_common::{OwnedDeviceId, OwnedEventId, OwnedRoomId, serde::StringEnum};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize, de, ser::SerializeStruct};

use crate::PrivOwnedStr;

/// The unstable `m.room.message` content field for an event stream descriptor.
pub const STREAM_DESCRIPTOR_KEY: &str = "org.matrix.msc4471.stream";

/// A descriptor declaring that a room event has a live event stream.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StreamDescriptor {
    /// The publisher device ID to which subscriptions should be sent.
    pub device_id: OwnedDeviceId,

    /// The lifetime of the descriptor in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_ms: Option<UInt>,
}

impl StreamDescriptor {
    /// Creates a new stream descriptor for the given publisher device.
    pub fn new(device_id: OwnedDeviceId) -> Self {
        Self { device_id, expiry_ms: None }
    }
}

/// The content of an `org.matrix.msc4471.stream.subscribe` to-device event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc4471.stream.subscribe", kind = ToDevice)]
pub struct ToDeviceStreamSubscribeEventContent {
    /// The room containing the stream descriptor.
    pub room_id: OwnedRoomId,

    /// The event containing the stream descriptor.
    pub event_id: OwnedEventId,

    /// The subscriber device which should receive updates.
    pub subscriber_device_id: OwnedDeviceId,

    /// Whether the subscriber requests a fresh `replace` baseline.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub resync: bool,
}

/// The content of an `org.matrix.msc4471.stream.subscribe` to-device event.
pub type StreamSubscribeEventContent = ToDeviceStreamSubscribeEventContent;

impl ToDeviceStreamSubscribeEventContent {
    /// Creates new stream subscription content.
    pub fn new(
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
        subscriber_device_id: OwnedDeviceId,
    ) -> Self {
        Self { room_id, event_id, subscriber_device_id, resync: false }
    }
}

/// The content of an `org.matrix.msc4471.stream.cancel` to-device event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc4471.stream.cancel", kind = ToDevice)]
pub struct ToDeviceStreamCancelEventContent {
    /// The room containing the stream descriptor.
    pub room_id: OwnedRoomId,

    /// The event containing the stream descriptor.
    pub event_id: OwnedEventId,

    /// The subscriber device whose subscription is cancelled.
    pub subscriber_device_id: OwnedDeviceId,

    /// A machine-readable reason for the cancellation.
    pub code: StreamCancelCode,

    /// A human-readable reason for debugging.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// The content of an `org.matrix.msc4471.stream.cancel` to-device event.
pub type StreamCancelEventContent = ToDeviceStreamCancelEventContent;

impl ToDeviceStreamCancelEventContent {
    /// Creates new stream cancellation content.
    pub fn new(
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
        subscriber_device_id: OwnedDeviceId,
        code: StreamCancelCode,
    ) -> Self {
        Self { room_id, event_id, subscriber_device_id, code, reason: None }
    }
}

/// Cancellation codes for event stream subscriptions.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all(prefix = "m.", rule = "snake_case"))]
#[non_exhaustive]
pub enum StreamCancelCode {
    /// The publisher device does not have an active stream for the descriptor.
    UnknownStream,

    /// The subscription request is malformed or names an invalid subscriber device.
    InvalidSubscription,

    /// The publisher device declined because the subscriber is not allowed to receive updates.
    Forbidden,

    /// The publisher device declined because of implementation limits.
    LimitExceeded,

    /// The subscriber device no longer wants updates for the stream.
    UserCancelled,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The content of an `org.matrix.msc4471.stream.update` to-device event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc4471.stream.update", kind = ToDevice)]
pub struct ToDeviceStreamUpdateEventContent {
    /// The room containing the stream descriptor.
    pub room_id: OwnedRoomId,

    /// The event containing the stream descriptor.
    pub event_id: OwnedEventId,

    /// A monotonically increasing sequence number for this subscriber's view.
    pub seq: UInt,

    /// The update operation.
    #[serde(flatten)]
    pub op: StreamUpdateOperation,
}

/// The content of an `org.matrix.msc4471.stream.update` to-device event.
pub type StreamUpdateEventContent = ToDeviceStreamUpdateEventContent;

impl ToDeviceStreamUpdateEventContent {
    /// Creates new stream update content.
    pub fn new(
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
        seq: UInt,
        op: StreamUpdateOperation,
    ) -> Self {
        Self { room_id, event_id, seq, op }
    }
}

/// An event stream update operation.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum StreamUpdateOperation {
    /// Replace the current transient `body`.
    Replace(StreamUpdateContent),

    /// Append text to the current transient `body`.
    Append(StreamUpdateContent),
}

impl Serialize for StreamUpdateOperation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("StreamUpdateOperation", 2)?;

        match self {
            Self::Replace(content) => {
                state.serialize_field("op", "replace")?;
                state.serialize_field("content", content)?;
            }
            Self::Append(content) => {
                state.serialize_field("op", "append")?;
                state.serialize_field("content", content)?;
            }
        }

        state.end()
    }
}

impl<'de> Deserialize<'de> for StreamUpdateOperation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            op: String,
            content: serde_json::Value,
        }

        let Helper { op, content } = Helper::deserialize(deserializer)?;

        match op.as_str() {
            "replace" => {
                serde_json::from_value(content).map(Self::Replace).map_err(de::Error::custom)
            }
            "append" => {
                serde_json::from_value(content).map(Self::Append).map_err(de::Error::custom)
            }
            _ => Err(de::Error::unknown_variant(&op, &["replace", "append"])),
        }
    }
}

/// The payload for an event stream update.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StreamUpdateContent {
    /// Text to replace or append to the current transient `body`.
    pub body: String,
}

impl StreamUpdateContent {
    /// Creates a new update payload.
    pub fn new(body: String) -> Self {
        Self { body }
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, owned_device_id, owned_event_id,
        owned_room_id, serde::Raw,
    };
    use serde_json::json;

    use super::{
        StreamCancelCode, StreamCancelEventContent, StreamDescriptor, StreamSubscribeEventContent,
        StreamUpdateContent, StreamUpdateEventContent, StreamUpdateOperation,
    };
    use crate::{
        AnyToDeviceEvent, ToDeviceEvent,
        room::message::{RoomMessageEventContent, RoomMessageEventContentWithoutRelation},
    };

    #[test]
    fn descriptor_serializes_in_room_message_content() {
        let mut content = RoomMessageEventContent::text_plain("Generating response...");
        content.stream = Some(StreamDescriptor {
            device_id: owned_device_id!("DEVICEID"),
            expiry_ms: Some(uint!(1_800_000)),
        });

        assert_to_canonical_json_eq!(
            content,
            json!({
                "msgtype": "m.text",
                "body": "Generating response...",
                "org.matrix.msc4471.stream": {
                    "device_id": "DEVICEID",
                    "expiry_ms": 1_800_000,
                },
            })
        );

        let raw = Raw::new(&content).unwrap();
        let deserialized: RoomMessageEventContent = raw.deserialize().unwrap();
        let stream = deserialized.stream.unwrap();
        assert_eq!(stream.device_id, "DEVICEID");
        assert_eq!(stream.expiry_ms, Some(uint!(1_800_000)));
    }

    #[test]
    fn replacement_omits_stream_descriptor() {
        let mut content = RoomMessageEventContent::text_plain("Generating response...");
        content.stream = Some(StreamDescriptor::new(owned_device_id!("DEVICEID")));

        let replacement = RoomMessageEventContentWithoutRelation::text_plain("Done");
        content.apply_replacement(replacement);

        assert!(content.stream.is_none());
        assert_to_canonical_json_eq!(
            content,
            json!({
                "msgtype": "m.text",
                "body": "Done",
            })
        );
    }

    #[test]
    fn subscribe_roundtrip() {
        let content = StreamSubscribeEventContent {
            room_id: owned_room_id!("!room:example.org"),
            event_id: owned_event_id!("$event:example.org"),
            subscriber_device_id: owned_device_id!("SUBSCRIBERDEVICE"),
            resync: true,
        };

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "subscriber_device_id": "SUBSCRIBERDEVICE",
                "resync": true,
            })
        );

        let raw = Raw::new(&content).unwrap();
        let deserialized: StreamSubscribeEventContent = raw.deserialize().unwrap();
        assert_eq!(deserialized.room_id, "!room:example.org");
        assert_eq!(deserialized.event_id, "$event:example.org");
        assert_eq!(deserialized.subscriber_device_id, "SUBSCRIBERDEVICE");
        assert!(deserialized.resync);
    }

    #[test]
    fn cancel_roundtrip() {
        let mut content = StreamCancelEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            owned_device_id!("SUBSCRIBERDEVICE"),
            StreamCancelCode::UnknownStream,
        );
        content.reason = Some("Unknown or expired stream".to_owned());

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "subscriber_device_id": "SUBSCRIBERDEVICE",
                "code": "m.unknown_stream",
                "reason": "Unknown or expired stream",
            })
        );

        let raw = Raw::new(&content).unwrap();
        let deserialized: StreamCancelEventContent = raw.deserialize().unwrap();
        assert_eq!(deserialized.subscriber_device_id, "SUBSCRIBERDEVICE");
        assert_eq!(deserialized.code, StreamCancelCode::UnknownStream);
        assert_eq!(
            serde_json::to_value(StreamCancelCode::UserCancelled).unwrap(),
            "m.user_cancelled"
        );
    }

    #[test]
    fn replace_update_roundtrip() {
        let content = StreamUpdateEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            uint!(1),
            StreamUpdateOperation::Replace(StreamUpdateContent::new(
                "The answer is still being generated.".to_owned(),
            )),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 1,
                "op": "replace",
                "content": {
                    "body": "The answer is still being generated.",
                },
            })
        );

        let raw = Raw::new(&content).unwrap();
        let deserialized: StreamUpdateEventContent = raw.deserialize().unwrap();
        assert_eq!(deserialized.seq, uint!(1));
        let StreamUpdateOperation::Replace(replacement) = deserialized.op else {
            panic!("expected replace");
        };
        assert_eq!(replacement.body, "The answer is still being generated.");
    }

    #[test]
    fn append_update_roundtrip() {
        let content = StreamUpdateEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            uint!(2),
            StreamUpdateOperation::Append(StreamUpdateContent::new(" Still working.".to_owned())),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 2,
                "op": "append",
                "content": {
                    "body": " Still working.",
                },
            })
        );

        let raw = Raw::new(&content).unwrap();
        let deserialized: StreamUpdateEventContent = raw.deserialize().unwrap();
        let StreamUpdateOperation::Append(append) = deserialized.op else {
            panic!("expected append");
        };
        assert_eq!(append.body, " Still working.");
    }

    #[test]
    fn to_device_event_enum_deserializes_stream_update() {
        let event = json!({
            "sender": "@alice:example.org",
            "type": "org.matrix.msc4471.stream.update",
            "content": {
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 1,
                "op": "replace",
                "content": {
                    "body": "hello",
                },
            },
        });

        let event = serde_json::from_value::<AnyToDeviceEvent>(event).unwrap();
        let AnyToDeviceEvent::StreamUpdate(ToDeviceEvent { content, .. }) = event else {
            panic!("expected stream update");
        };
        let StreamUpdateOperation::Replace(replacement) = content.op else {
            panic!("expected replace");
        };
        assert_eq!(replacement.body, "hello");
    }
}
