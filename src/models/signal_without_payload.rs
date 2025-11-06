use sea_orm::FromQueryResult;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromQueryResult)]
pub struct SignalWithoutPayload {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub provider_slug: String,
    pub connection_id: Uuid,
    pub kind: String,
    pub occurred_at: DateTimeWithTimeZone,
    pub received_at: DateTimeWithTimeZone,
    pub dedupe_key: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}
