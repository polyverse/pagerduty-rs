use serde::Serialize;
use serde::Serializer;
use std::convert::From;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::Duration;
use time::OffsetDateTime;

const CONTENT_TYPE: &str = "content-type";
const USER_AGENT: &str = "user-agent";
const CONTENT_ENCODING: &str = "content-encoding";

const CONTENT_ENCODING_IDENTITY: &str = "identity";
const CONTENT_TYPE_JSON: &str = "application/json";

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

/// Private Change payload
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

/// Private Change serialization structure.
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

/// Private Change serialization structure.
#[derive(Serialize)]
struct SendableChange<T: Serialize> {
    /// This is the 32 character Integration Key for an integration on a service or on a global ruleset.
    /// Set to None to have PagerDuty sender fill it in.
    routing_key: String,

    /// Payload for the change event
    payload: ChangePayload<T>,

    /// List of links to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<Links>,
}

#[derive(Serialize)]
struct SendableAlertTrigger<T: Serialize> {
    /// This is the 32 character Integration Key for an integration on a service or on a global ruleset.
    /// Set to None to have PagerDuty sender fill it in.
    routing_key: String,

    payload: AlertTriggerPayload<T>,

    /// Deduplication key for correlating triggers and resolves. The maximum permitted length of this
    /// property is 255 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    dedup_key: Option<String>,

    /// List of images to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Images>,

    /// List of links to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<Links>,

    /// The type of event. Can be trigger, acknowledge or resolve.
    event_action: Action,

    /// Name of the client creating this event
    #[serde(skip_serializing_if = "Option::is_none")]
    client: Option<String>,

    /// URL of the client's homepage/service/whatever.
    #[serde(skip_serializing_if = "Option::is_none")]
    client_url: Option<String>,
}

#[derive(Serialize)]
struct SendableAlertFollowup {
    routing_key: String,
    dedup_key: String,
    event_action: Action,
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

fn datetime_to_iso8601<S>(d: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    //serializer.serialize_str(d.format(Format::Rfc3339).as_str())
    serializer.serialize_str(d.format("%FT%T.%NZ").as_str())
}

#[derive(Debug)]
pub enum EventsV2Error {
    SerdeJsonError(serde_json::Error),

    /// A synthetic error from ureq library (basically not an HTTP error of any kind.)
    SyntheticUreqError(Option<ureq::Error>),

    /// Any Http Error
    HttpError(u16),

    /// We expected PagerDuty to respond with 202 Accepted - this indicates we got anything but 202
    /// could be 1xx or 3xx codes as well.
    NotAccepted(u16),
}

impl Error for EventsV2Error {}
impl Display for EventsV2Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::SerdeJsonError(e) => write!(f, "SerdeJsonError: {}", e),
            Self::SyntheticUreqError(oe) => match oe {
                Some(e) => write!(f, "SyntheticUreqError: {}", e),
                None => write!(f, "SyntheticUreqError (no details)"),
            },
            Self::HttpError(c) => write!(f, "HttpError with Status Code {}", c),
            Self::NotAccepted(c) => {
                write!(f, "Http Status Code was other than 202 (Accepted): {}", c)
            }
        }
    }
}
impl From<serde_json::Error> for EventsV2Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJsonError(err)
    }
}

pub type EventsV2Result = Result<(), EventsV2Error>;

/// The main PagerDuty Events V2 API
pub struct EventsV2 {
    /// The integration/routing key for a generated PagerDuty service
    integration_key: String,
    user_agent: Option<String>,
}

impl EventsV2 {
    pub fn new(
        integration_key: String,
        user_agent: Option<String>,
    ) -> Result<EventsV2, EventsV2Error> {
        Ok(EventsV2 {
            integration_key,
            user_agent,
        })
    }

    pub fn event<T: Serialize>(&self, event: Event<T>) -> EventsV2Result {
        match event {
            Event::Change(c) => self.change(c),
            Event::AlertTrigger(at) => self.alert_trigger(at),
            Event::AlertAcknowledge(aa) => self.alert_followup(aa.dedup_key, Action::Acknowledge),
            Event::AlertResolve(ar) => self.alert_followup(ar.dedup_key, Action::Resolve),
        }
    }

    fn change<T: Serialize>(&self, change: Change<T>) -> EventsV2Result {
        let sendable_change = SendableChange {
            routing_key: self.integration_key.clone(),
            links: change.links,
            payload: change.payload,
        };

        self.do_post(
            "https://events.pagerduty.com/v2/change/enqueue",
            sendable_change,
        )
    }

    fn alert_trigger<T: Serialize>(&self, alert_trigger: AlertTrigger<T>) -> EventsV2Result {
        let sendable_alert_trigger = SendableAlertTrigger {
            routing_key: self.integration_key.clone(),
            event_action: Action::Trigger,
            dedup_key: alert_trigger.dedup_key,
            images: alert_trigger.images,
            links: alert_trigger.links,
            payload: alert_trigger.payload,
            client: alert_trigger.client,
            client_url: alert_trigger.client_url,
        };

        self.do_post(
            "https://events.pagerduty.com/v2/enqueue",
            sendable_alert_trigger,
        )
    }

    fn alert_followup(&self, dedup_key: String, action: Action) -> EventsV2Result {
        let sendable_alert_followup = SendableAlertFollowup {
            routing_key: self.integration_key.clone(),
            event_action: action,
            dedup_key,
        };

        self.do_post(
            "https://events.pagerduty.com/v2/enqueue",
            sendable_alert_followup,
        )
    }

    // Make this part Async in the future
    fn do_post<T: Serialize>(&self, url: &str, content: T) -> EventsV2Result {
        let jsonstr = serde_json::to_string(&content)?;

        let mut request = ureq::post(url);

        request
            .set(CONTENT_TYPE, CONTENT_TYPE_JSON)
            .set(CONTENT_ENCODING, CONTENT_ENCODING_IDENTITY)
            // 300 seconds should be plenty to post to pagerduty
            .timeout(Duration::from_secs(300));

        if let Some(ua) = &self.user_agent {
            request.set(USER_AGENT, ua);
        };

        let resp = request.send_string(jsonstr.as_str());

        if resp.synthetic() {
            return Err(EventsV2Error::SyntheticUreqError(
                resp.into_synthetic_error(),
            ));
        }

        if resp.error() {
            return Err(EventsV2Error::HttpError(resp.status()));
        }

        if resp.status() != 202 {
            return Err(EventsV2Error::NotAccepted(resp.status()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(Serialize)]
    pub struct SerializableTest {
        some_field: String,
        another_field: usize,
    }

    #[test]
    fn serialize_change() {
        // With everything optional
        let c = Change {
            payload: ChangePayload {
                summary: "Hello".to_owned(),
                source: Some("hostname".to_owned()),
                timestamp: OffsetDateTime::from_unix_timestamp_nanos(2000071804323000000),
                custom_details: Some(SerializableTest {
                    some_field: "Serialize this!".to_owned(),
                    another_field: 34,
                }),
            },
            links: Some(vec![Link {
                href: "https://polyverse.com".to_owned(),
                text: Some("Polyverse homepage".to_owned()),
            }]),
        };

        let cr = serde_json::to_string(&c);
        assert!(cr.is_ok());
        assert_eq!(cr.unwrap(), "{\"payload\":{\"summary\":\"Hello\",\"timestamp\":\"2033-05-18T23:30:04.323000000Z\",\"source\":\"hostname\",\"custom_details\":{\"some_field\":\"Serialize this!\",\"another_field\":34}},\"links\":[{\"href\":\"https://polyverse.com\",\"text\":\"Polyverse homepage\"}]}");

        // With nothing optional
        let c = Change::<()> {
            payload: ChangePayload {
                summary: "Hello".to_owned(),
                timestamp: OffsetDateTime::from_unix_timestamp_nanos(2000071804323000000),
                source: None,
                custom_details: None,
            },
            links: None,
        };

        let cr = serde_json::to_string(&c);
        assert!(cr.is_ok());
        assert_eq!(
            cr.unwrap(),
            "{\"payload\":{\"summary\":\"Hello\",\"timestamp\":\"2033-05-18T23:30:04.323000000Z\"}}"
        );
    }

    #[test]
    fn serialize_sendable_change() {
        // With everything optional
        let c = SendableChange {
            routing_key: "routingkey".to_owned(),
            payload: ChangePayload {
                summary: "Hello".to_owned(),
                source: Some("hostname".to_owned()),
                timestamp: OffsetDateTime::from_unix_timestamp_nanos(2000071804323000000),
                custom_details: Some(SerializableTest {
                    some_field: "Serialize this!".to_owned(),
                    another_field: 34,
                }),
            },
            links: Some(vec![Link {
                href: "https://polyverse.com".to_owned(),
                text: Some("Polyverse homepage".to_owned()),
            }]),
        };

        let cr = serde_json::to_string(&c);
        assert!(cr.is_ok());
        assert_eq!(cr.unwrap(), "{\"routing_key\":\"routingkey\",\"payload\":{\"summary\":\"Hello\",\"timestamp\":\"2033-05-18T23:30:04.323000000Z\",\"source\":\"hostname\",\"custom_details\":{\"some_field\":\"Serialize this!\",\"another_field\":34}},\"links\":[{\"href\":\"https://polyverse.com\",\"text\":\"Polyverse homepage\"}]}");

        // With nothing optional
        let c = SendableChange::<()> {
            routing_key: "routingkey".to_owned(),
            payload: ChangePayload {
                summary: "Hello".to_owned(),
                timestamp: OffsetDateTime::from_unix_timestamp_nanos(2000071804323000000),
                source: None,
                custom_details: None,
            },
            links: None,
        };

        let cr = serde_json::to_string(&c);
        assert!(cr.is_ok());
        assert_eq!(cr.unwrap(), "{\"routing_key\":\"routingkey\",\"payload\":{\"summary\":\"Hello\",\"timestamp\":\"2033-05-18T23:30:04.323000000Z\"}}");
    }

    #[test]
    fn serialize_alert_trigger() {
        // With everything optional
        let a = AlertTrigger {
            payload: AlertTriggerPayload {
                summary: "Hello".to_owned(),
                source: "hostname".to_owned(),
                timestamp: Some(OffsetDateTime::from_unix_timestamp_nanos(
                    2000071804323000000,
                )),
                severity: Severity::Info,
                component: Some("postgres".to_owned()),
                group: Some("prod-datapipe".to_owned()),
                class: Some("deploy".to_owned()),
                custom_details: Some(SerializableTest {
                    some_field: "Serialize this!".to_owned(),
                    another_field: 34,
                }),
            },
            dedup_key: Some("dedupkey1".to_owned()),
            images: Some(vec![Image {
                src: "https://polyverse.com/static/img/SplashPageIMG/polyverse_blue.png".to_owned(),
                href: Some("https://polyverse.com".to_owned()),
                alt: Some("The Polyverse Logo".to_owned()),
            }]),
            links: Some(vec![Link {
                href: "https://polyverse.com".to_owned(),
                text: Some("Polyverse homepage".to_owned()),
            }]),
            client: Some("Zerotect".to_owned()),
            client_url: Some("https://github.com/polyverse/zerotect".to_owned()),
        };

        let ar = serde_json::to_string(&a);
        assert!(ar.is_ok());
        assert_eq!(ar.unwrap(), "{\"payload\":{\"severity\":\"info\",\"summary\":\"Hello\",\"source\":\"hostname\",\"timestamp\":\"2033-05-18T23:30:04.323000000Z\",\"component\":\"postgres\",\"group\":\"prod-datapipe\",\"class\":\"deploy\",\"custom_details\":{\"some_field\":\"Serialize this!\",\"another_field\":34}},\"dedup_key\":\"dedupkey1\",\"images\":[{\"src\":\"https://polyverse.com/static/img/SplashPageIMG/polyverse_blue.png\",\"href\":\"https://polyverse.com\",\"alt\":\"The Polyverse Logo\"}],\"links\":[{\"href\":\"https://polyverse.com\",\"text\":\"Polyverse homepage\"}],\"client\":\"Zerotect\",\"client_url\":\"https://github.com/polyverse/zerotect\"}");

        // With nothing optional
        let a = AlertTrigger::<()> {
            payload: AlertTriggerPayload {
                summary: "Hello".to_owned(),
                source: "hostname".to_owned(),
                timestamp: None,
                severity: Severity::Info,
                component: None,
                group: None,
                class: None,
                custom_details: None,
            },
            dedup_key: None,
            images: None,
            links: None,
            client: None,
            client_url: None,
        };

        let ar = serde_json::to_string(&a);
        assert!(ar.is_ok());
        assert_eq!(
            ar.unwrap(),
            "{\"payload\":{\"severity\":\"info\",\"summary\":\"Hello\",\"source\":\"hostname\"}}"
        );
    }

    #[test]
    fn serialize_sendable_alert_trigger() {
        // With everything optional
        let a = SendableAlertTrigger {
            routing_key: "routingkey".to_owned(),
            event_action: Action::Trigger,
            payload: AlertTriggerPayload {
                summary: "Hello".to_owned(),
                source: "hostname".to_owned(),
                timestamp: Some(OffsetDateTime::from_unix_timestamp_nanos(
                    2000071804323000000,
                )),
                severity: Severity::Info,
                component: Some("postgres".to_owned()),
                group: Some("prod-datapipe".to_owned()),
                class: Some("deploy".to_owned()),
                custom_details: Some(SerializableTest {
                    some_field: "Serialize this!".to_owned(),
                    another_field: 34,
                }),
            },
            dedup_key: Some("dedupkey1".to_owned()),
            images: Some(vec![Image {
                src: "https://polyverse.com/static/img/SplashPageIMG/polyverse_blue.png".to_owned(),
                href: Some("https://polyverse.com".to_owned()),
                alt: Some("The Polyverse Logo".to_owned()),
            }]),
            links: Some(vec![Link {
                href: "https://polyverse.com".to_owned(),
                text: Some("Polyverse homepage".to_owned()),
            }]),
            client: Some("Zerotect".to_owned()),
            client_url: Some("https://github.com/polyverse/zerotect".to_owned()),
        };

        let ar = serde_json::to_string(&a);
        assert!(ar.is_ok());
        assert_eq!(ar.unwrap(), "{\"routing_key\":\"routingkey\",\"payload\":{\"severity\":\"info\",\"summary\":\"Hello\",\"source\":\"hostname\",\"timestamp\":\"2033-05-18T23:30:04.323000000Z\",\"component\":\"postgres\",\"group\":\"prod-datapipe\",\"class\":\"deploy\",\"custom_details\":{\"some_field\":\"Serialize this!\",\"another_field\":34}},\"dedup_key\":\"dedupkey1\",\"images\":[{\"src\":\"https://polyverse.com/static/img/SplashPageIMG/polyverse_blue.png\",\"href\":\"https://polyverse.com\",\"alt\":\"The Polyverse Logo\"}],\"links\":[{\"href\":\"https://polyverse.com\",\"text\":\"Polyverse homepage\"}],\"event_action\":\"trigger\",\"client\":\"Zerotect\",\"client_url\":\"https://github.com/polyverse/zerotect\"}");

        // With nothing optional
        let a = SendableAlertTrigger::<()> {
            routing_key: "routingkey".to_owned(),
            event_action: Action::Trigger,
            payload: AlertTriggerPayload {
                summary: "Hello".to_owned(),
                source: "hostname".to_owned(),
                timestamp: None,
                severity: Severity::Info,
                component: None,
                group: None,
                class: None,
                custom_details: None,
            },
            dedup_key: None,
            images: None,
            links: None,
            client: None,
            client_url: None,
        };

        let ar = serde_json::to_string(&a);
        assert!(ar.is_ok());
        assert_eq!(ar.unwrap(), "{\"routing_key\":\"routingkey\",\"payload\":{\"severity\":\"info\",\"summary\":\"Hello\",\"source\":\"hostname\"},\"event_action\":\"trigger\"}");
    }

    #[test]
    fn serialize_alert_acknowledge() {
        let a = AlertAcknowledge {
            dedup_key: "dedupkeyacknowledge".to_owned(),
        };

        let ar = serde_json::to_string(&a);
        assert!(ar.is_ok());
        assert_eq!(ar.unwrap(), "{\"dedup_key\":\"dedupkeyacknowledge\"}");
    }

    #[test]
    fn serialize_alert_resolve() {
        let a = AlertResolve {
            dedup_key: "dedupkeyacknowledge".to_owned(),
        };

        let ar = serde_json::to_string(&a);
        assert!(ar.is_ok());
        assert_eq!(ar.unwrap(), "{\"dedup_key\":\"dedupkeyacknowledge\"}");
    }

    #[test]
    fn serialize_sendable_alert_followup() {
        let ss = SendableAlertFollowup {
            dedup_key: "DedupkeyFollowup".to_owned(),
            routing_key: "routingkey".to_owned(),
            event_action: Action::Resolve,
        };

        let ssr = serde_json::to_string(&ss);
        assert!(ssr.is_ok());
        assert_eq!(ssr.unwrap(), "{\"routing_key\":\"routingkey\",\"dedup_key\":\"DedupkeyFollowup\",\"event_action\":\"resolve\"}");
    }
}
