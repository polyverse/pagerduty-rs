use crate::types::*;

use serde::Serialize;

/// Private Change serialization structure.
#[derive(Serialize)]
pub struct SendableChange<T: Serialize> {
    /// This is the 32 character Integration Key for an integration on a service or on a global ruleset.
    /// Set to None to have PagerDuty sender fill it in.
    pub routing_key: String,

    /// Payload for the change event
    pub payload: ChangePayload<T>,

    /// List of links to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Serialize)]
pub struct SendableAlertTrigger<T: Serialize> {
    /// This is the 32 character Integration Key for an integration on a service or on a global ruleset.
    /// Set to None to have PagerDuty sender fill it in.
    pub routing_key: String,

    pub payload: AlertTriggerPayload<T>,

    /// Deduplication key for correlating triggers and resolves. The maximum permitted length of this
    /// property is 255 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedup_key: Option<String>,

    /// List of images to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Images>,

    /// List of links to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,

    /// The type of event. Can be trigger, acknowledge or resolve.
    pub event_action: Action,

    /// Name of the client creating this event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,

    /// URL of the client's homepage/service/whatever.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_url: Option<String>,
}

#[derive(Serialize)]
pub struct SendableAlertFollowup {
    pub routing_key: String,
    pub dedup_key: String,
    pub event_action: Action,
}
