use base64::Engine;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{database::HasValueRef, Decode, Postgres, TypeInfo, ValueRef as _};
use uuid::Uuid;

use crate::var::Var;

pub trait Iterable {
    fn cursor(&self) -> Cursor;
}

impl<T> Iterable for Option<T>
where
    T: Default + Iterable,
{
    fn cursor(&self) -> Cursor {
        match self {
            Some(v) => v.cursor(),
            None => T::default().cursor(),
        }
    }
}

impl Iterable for i32 {
    fn cursor(&self) -> Cursor {
        Cursor::I32
    }
}

impl Iterable for String {
    fn cursor(&self) -> Cursor {
        Cursor::String
    }
}

impl Iterable for Uuid {
    fn cursor(&self) -> Cursor {
        Cursor::Uuid
    }
}

impl Iterable for DateTime<Utc> {
    fn cursor(&self) -> Cursor {
        Cursor::DateTime
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Cursor {
    I32,
    String,
    Uuid,
    DateTime,
}

impl Cursor {
    pub fn infer(column: <Postgres as HasValueRef<'_>>::ValueRef) -> sqlx::Result<String> {
        Ok(match column.type_info().as_ref().name() {
            "INT" | "INTEGER" => I32Cursor::encode(
                &<i32 as Decode<'_, Postgres>>::decode(column).map_err(sqlx::Error::Decode)?,
            ),
            "TEXT" | "VARCHAR" => StringCursor::encode(
                &<String as Decode<'_, Postgres>>::decode(column).map_err(sqlx::Error::Decode)?,
            ),
            "UUID" => UuidCursor::encode(
                &<Uuid as Decode<'_, Postgres>>::decode(column).map_err(sqlx::Error::Decode)?,
            ),
            "TIMESTAMP" | "TIMESTAMPTZ" => DateTimeCursor::encode(
                &<DateTime<Utc> as Decode<'_, Postgres>>::decode(column)
                    .map_err(sqlx::Error::Decode)?,
            ),
            x => {
                return Err(sqlx::Error::Decode(
                    format!("invalid cursor type during inference: {}", x).into(),
                ))
            }
        })
    }

    pub fn decode(&self, encoded: &str) -> Var {
        match self {
            Self::I32 => Var::I32(I32Cursor::decode(encoded)),
            Self::String => Var::String(StringCursor::decode(encoded)),
            Self::Uuid => Var::Uuid(UuidCursor::decode(encoded)),
            Self::DateTime => Var::DateTime(DateTimeCursor::decode(encoded)),
        }
    }

    pub fn encode(literal: &Var) -> String {
        match literal {
            Var::Bool(_) => panic!("invalid cursor type: bool"),
            Var::I32(v) => I32Cursor::encode(v),
            Var::String(v) => StringCursor::encode(v),
            Var::Uuid(v) => UuidCursor::encode(v),
            Var::DateTime(v) => DateTimeCursor::encode(v),
        }
    }

    pub fn min(self) -> Var {
        match self {
            Self::I32 => Var::I32(I32Cursor::min()),
            Self::String => Var::String(StringCursor::min()),
            Self::Uuid => Var::Uuid(UuidCursor::min()),
            Self::DateTime => Var::DateTime(DateTimeCursor::min()),
        }
    }

    pub fn max(self) -> Var {
        match self {
            Self::I32 => Var::I32(I32Cursor::max()),
            Self::String => Var::String(StringCursor::max()),
            Self::Uuid => Var::Uuid(UuidCursor::max()),
            Self::DateTime => Var::DateTime(DateTimeCursor::max()),
        }
    }
}

impl From<I32Cursor> for Cursor {
    fn from(_cursor: I32Cursor) -> Self {
        Self::I32
    }
}

impl From<StringCursor> for Cursor {
    fn from(_cursor: StringCursor) -> Self {
        Self::String
    }
}

impl From<UuidCursor> for Cursor {
    fn from(_cursor: UuidCursor) -> Self {
        Self::Uuid
    }
}

impl From<DateTimeCursor> for Cursor {
    fn from(_cursor: DateTimeCursor) -> Self {
        Self::DateTime
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct I32Cursor;

impl I32Cursor {
    pub fn new() -> Self {
        Self
    }

    pub fn decode(encoded: &str) -> i32 {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|buf| buf.as_slice().try_into().ok())
            .map(i32::from_be_bytes)
            .unwrap_or_else(|| {
                tracing::warn!("invalid i32 cursor '{}'", encoded);
                Self::min()
            })
    }

    pub fn encode(decoded: &i32) -> String {
        base64::engine::general_purpose::STANDARD.encode(decoded.to_be_bytes())
    }

    pub fn min() -> i32 {
        i32::MIN
    }

    pub fn max() -> i32 {
        i32::MAX
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StringCursor;

impl StringCursor {
    pub fn new() -> Self {
        Self
    }

    pub fn decode(encoded: &str) -> String {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|buf| String::from_utf8(buf.as_slice().to_vec()).ok())
            .unwrap_or_else(|| {
                tracing::warn!("invalid string cursor '{}'", encoded);
                Self::min()
            })
    }

    pub fn encode(decoded: &String) -> String {
        base64::engine::general_purpose::STANDARD.encode(decoded.as_bytes())
    }

    pub fn min() -> String {
        "".to_string()
    }

    #[allow(dead_code)]
    pub fn max() -> String {
        "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~".to_string()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct UuidCursor;

impl UuidCursor {
    pub fn new() -> Self {
        Self
    }

    pub fn decode(encoded: &str) -> Uuid {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|buf| buf.as_slice().try_into().ok())
            .map(Uuid::from_bytes)
            .unwrap_or_else(|| {
                tracing::warn!("invalid uuid cursor '{}'", encoded);
                Self::min()
            })
    }

    pub fn encode(decoded: &Uuid) -> String {
        base64::engine::general_purpose::STANDARD.encode(decoded.as_bytes())
    }

    pub fn min() -> Uuid {
        Uuid::from_bytes([0; 16])
    }

    pub fn max() -> Uuid {
        Uuid::from_bytes([255; 16])
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DateTimeCursor;

impl DateTimeCursor {
    pub fn new() -> Self {
        Self
    }

    pub fn decode(encoded: &str) -> DateTime<Utc> {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|buf| buf.as_slice().try_into().ok())
            .map(|buf| Utc.timestamp_nanos(i64::from_be_bytes(buf)))
            .unwrap_or_else(|| {
                tracing::warn!("invalid datetime cursor '{}'", encoded);
                Self::min()
            })
    }

    pub fn encode(decoded: &DateTime<Utc>) -> String {
        base64::engine::general_purpose::STANDARD.encode(
            decoded
                .timestamp_nanos_opt()
                .expect("timestamp must be valid")
                .to_be_bytes(),
        )
    }

    pub fn min() -> DateTime<Utc> {
        Utc.timestamp_nanos(i64::MIN)
    }

    pub fn max() -> DateTime<Utc> {
        Utc.timestamp_nanos(i64::MAX)
    }
}
