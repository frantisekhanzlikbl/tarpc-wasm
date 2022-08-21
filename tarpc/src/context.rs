// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

//! Provides a request context that carries a deadline and trace context. This context is sent from
//! client to server and is used by the server to enforce response deadlines.
use static_assertions::assert_impl_all;
use std::time::Duration;
use time::OffsetDateTime;

/// A request context that carries request-scoped information like deadlines and trace information.
/// It is sent from client to server and is used by the server to enforce response deadlines.
///
/// The context should not be stored directly in a server implementation, because the context will
/// be different for each request in scope.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Context {
    /// When the client expects the request to be complete by. The server should cancel the request
    /// if it is not complete by this time.
    #[cfg_attr(feature = "serde1", serde(default = "ten_seconds_from_now"))]
    // Serialized as a Duration to prevent clock skew issues.
    #[cfg_attr(feature = "serde1", serde(with = "absolute_to_relative_time"))]
    pub deadline: OffsetDateTime,
}

#[cfg(feature = "serde1")]
mod absolute_to_relative_time {
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub use std::time::Duration;
    use time::OffsetDateTime;

    pub fn serialize<S>(deadline: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let deadline = *deadline - OffsetDateTime::now_utc();
        deadline.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deadline = Duration::deserialize(deserializer)?;
        Ok(OffsetDateTime::now_utc() + deadline)
    }

    #[cfg(test)]
    #[derive(serde::Serialize, serde::Deserialize)]
    struct AbsoluteToRelative(#[serde(with = "self")] OffsetDateTime);

    #[test]
    fn test_serialize() {
        let now = OffsetDateTime::now_utc();
        let deadline = now + Duration::from_secs(10);
        let serialized_deadline = bincode::serialize(&AbsoluteToRelative(deadline)).unwrap();
        let deserialized_deadline: Duration = bincode::deserialize(&serialized_deadline).unwrap();
        // TODO: how to avoid flakiness?
        assert!(deserialized_deadline > Duration::from_secs(9));
    }

    #[test]
    fn test_deserialize() {
        let deadline = Duration::from_secs(10);
        let serialized_deadline = bincode::serialize(&deadline).unwrap();
        let AbsoluteToRelative(deserialized_deadline) =
            bincode::deserialize(&serialized_deadline).unwrap();
        // TODO: how to avoid flakiness?
        assert!(deserialized_deadline > OffsetDateTime::now_utc() + Duration::from_secs(9));
    }
}

assert_impl_all!(Context: Send, Sync);

fn ten_seconds_from_now() -> OffsetDateTime {
    OffsetDateTime::now_utc() + Duration::from_secs(10)
}

/// Returns the context for the current request, or a default Context if no request is active.
pub fn current() -> Context {
    Context::current()
}

#[derive(Clone)]
struct Deadline(OffsetDateTime);

impl Default for Deadline {
    fn default() -> Self {
        Self(ten_seconds_from_now())
    }
}

impl Context {
    /// Returns the context for the current request, or a default Context if no request is active.
    pub fn current() -> Self {
        Self {
            deadline: Deadline::default().0,
        }
    }
}
