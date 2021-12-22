use serde::{ser::Error as SerializeError, Serialize, Serializer};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// Indicates the severity of the impact to the affected system.
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Serialize)]
pub struct Link {
    /// URL of the link to be attached.
    pub href: String,

    /// Plain text that describes the purpose of the link, and can be used as the link's text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

pub type Links = Vec<Link>;

#[derive(Serialize)]
pub struct Image {
    /// The source (URL) of the image being attached to the incident. This image must be served via HTTPS.
    pub src: String,

    /// Optional URL; makes the image a clickable link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,

    /// Optional alternative text for the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
}

pub type Images = Vec<Image>;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// A new alert is opened or a trigger log entry is created on an existing alert if one already
    /// exists with the same dedup_key.
    ///
    /// Use this event action when a new problem has been detected. Additional triggers may be sent
    /// when a previously detected problem has occurred again.
    Trigger,

    /// The incident referenced with the dedup_key will enter the acknowledged state.
    ///
    /// While an incident is acknowledged, it won't generate any additional notifications,
    /// even if it receives new trigger events.
    ///
    /// Use this event action to indicate that someone is presently working on the problem.
    Acknowledge,

    /// The incident referenced with the dedup_key will enter the resolved state.
    ///
    /// Once an incident is resolved, it won't generate any additional notifications. New trigger
    /// events with the same incident_key / dedup_keyas a resolved incident won't re-open
    /// the incident. Instead, a new incident will be created.
    ///
    /// Use this event action when the problem that caused the initial trigger event has been fixed.
    Resolve,
}

/// Change payload
#[derive(Serialize)]
pub struct ChangePayload<T: Serialize> {
    /// A brief text summary of the event. Displayed in PagerDuty to provide information about the change.
    /// The maximum permitted length of this property is 1024 characters.
    pub summary: String,

    /// The time at which the emitting tool detected or generated the event.
    #[serde(serialize_with = "datetime_to_iso8601")]
    pub timestamp: OffsetDateTime,

    /// The unique name of the location where the Change Event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Additional details about the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_details: Option<T>,
}

/// Change serialization structure.
#[derive(Serialize)]
pub struct Change<T: Serialize> {
    /// Payload for the change event
    pub payload: ChangePayload<T>,

    /// List of links to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Serialize)]
pub struct AlertTriggerPayload<T: Serialize> {
    /// The perceived severity of the status the event is describing with respect to the affected system.
    /// This can be critical, error, warning or info.
    pub severity: Severity,

    /// A brief text summary of the event, used to generate the summaries/titles of any associated alerts.
    /// The maximum permitted length of this property is 1024 characters.
    pub summary: String,

    /// The unique location of the affected system, preferably a hostname or FQDN.
    pub source: String,

    /// The time at which the emitting tool detected or generated the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "optional_datetime_to_iso8601")]
    pub timestamp: Option<OffsetDateTime>,

    /// Component of the source machine that is responsible for the event, for example mysql or eth0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,

    /// Logical grouping of components of a service, for example app-stack
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    /// The class/type of the event, for example ping failure or cpu load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,

    /// Additional details about the event and affected system
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_details: Option<T>,
}

#[derive(Serialize)]
pub struct AlertTrigger<T: Serialize> {
    /// The payload for this alert
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

    /// Name of the client creating this event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,

    /// URL of the client's homepage/service/whatever.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_url: Option<String>,
}

#[derive(Serialize)]
pub struct AlertAcknowledge {
    pub dedup_key: String,
}

#[derive(Serialize)]
pub struct AlertResolve {
    pub dedup_key: String,
}

pub enum Event<T: Serialize> {
    Change(Change<T>),
    AlertTrigger(AlertTrigger<T>),
    AlertAcknowledge(AlertAcknowledge),
    AlertResolve(AlertResolve),
}

fn optional_datetime_to_iso8601<S>(
    od: &Option<OffsetDateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match od.as_ref() {
        Some(d) => datetime_to_iso8601(d, serializer),
        None => serializer.serialize_none(),
    }
}

// This suggestion
fn datetime_to_iso8601<S>(d: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match d.format(&Rfc3339) {
        Ok(formatted) => serializer.serialize_str(formatted.as_str()),
        Err(e) => Err(SerializeError::custom(format!("{}", e))),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use time::macros::date;

    #[test]
    fn test_serialization_pads() {
        let change = Change {
            payload: ChangePayload::<()> {
                summary: "Testing timestamp serialization".to_owned(),
                timestamp: date!(2021 - 05 - 30).midnight().assume_utc(),
                source: None,
                custom_details: None,
            },

            links: None,
        };

        assert_eq!("{\"payload\":{\"summary\":\"Testing timestamp serialization\",\"timestamp\":\"2021-05-30T00:00:00Z\"}}", serde_json::to_string(&change).unwrap());
    }
}
