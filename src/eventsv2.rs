use chrono::{DateTime, Utc};
use serde::Serialize;
use std::convert::From;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::Duration;

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
    href: String,

    /// Plain text that describes the purpose of the link, and can be used as the link's text.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

pub type Links = Vec<Link>;

#[derive(Serialize)]
pub struct Image {
    /// The source (URL) of the image being attached to the incident. This image must be served via HTTPS.
    src: String,

    /// Optional URL; makes the image a clickable link.
    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>,

    /// Optional alternative text for the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    alt: Option<String>,
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
    summary: String,

    /// The unique name of the location where the Change Event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,

    /// The time at which the emitting tool detected or generated the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<DateTime<Utc>>,

    /// Additional details about the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_details: Option<T>,
}

/// Private Change serialization structure.
#[derive(Serialize)]
pub struct Change<T: Serialize> {
    /// This is the 32 character Integration Key for an integration on a service or on a global ruleset.
    /// Set to None to have PagerDuty sender fill it in.
    #[serde(skip_serializing_if = "Option::is_none")]
    routing_key: Option<String>,

    /// Payload for the change event
    payload: ChangePayload<T>,

    /// List of links to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<Links>,
}

#[derive(Serialize)]
pub struct AlertPayload<T: Serialize> {
    /// The perceived severity of the status the event is describing with respect to the affected system.
    /// This can be critical, error, warning or info.
    severity: Severity,

    /// A brief text summary of the event, used to generate the summaries/titles of any associated alerts.
    /// The maximum permitted length of this property is 1024 characters.
    summary: String,

    /// The unique location of the affected system, preferably a hostname or FQDN.
    source: String,

    /// The time at which the emitting tool detected or generated the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<DateTime<Utc>>,

    /// Component of the source machine that is responsible for the event, for example mysql or eth0
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<String>,

    /// Logical grouping of components of a service, for example app-stack
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,

    /// The class/type of the event, for example ping failure or cpu load
    #[serde(skip_serializing_if = "Option::is_none")]
    class: Option<String>,

    /// Additional details about the event and affected system
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_details: Option<T>,
}

#[derive(Serialize)]
pub struct Alert<T: Serialize> {
    /// This is the 32 character Integration Key for an integration on a service or on a global ruleset.
    /// Set to None to have PagerDuty sender fill it in.
    #[serde(skip_serializing_if = "Option::is_none")]
    routing_key: Option<String>,

    payload: AlertPayload<T>,

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

pub enum Event<T: Serialize> {
    Change(Change<T>),
    Alert(Alert<T>),
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
        write!(f, "PagerDutyError")
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
            Event::Alert(a) => self.alert(a),
        }
    }

    fn change<T: Serialize>(&self, change: Change<T>) -> EventsV2Result {
        let sanitized_change = match change.routing_key {
            Some(_) => change,
            None => Change::<T> {
                routing_key: Some(self.integration_key.clone()),
                links: change.links,
                payload: change.payload,
            },
        };

        self.do_post(
            "https://events.pagerduty.com/v2/change/enqueue",
            sanitized_change,
        )
    }

    fn alert<T: Serialize>(&self, alert: Alert<T>) -> EventsV2Result {
        let sanitized_alert = match alert.routing_key {
            Some(_) => alert,
            None => Alert::<T> {
                routing_key: Some(self.integration_key.clone()),
                event_action: alert.event_action,
                dedup_key: alert.dedup_key,
                images: alert.images,
                links: alert.links,
                payload: alert.payload,
                client: alert.client,
                client_url: alert.client_url,
            },
        };

        self.do_post(
            "https://events.pagerduty.com/v2/change/enqueue",
            sanitized_alert,
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

/*
fn endpoint(event_type: EventType) -> &str {
    match event_type {
        EventType::Alert => "https://events.pagerduty.com/v2/enqueue",
        EventType::Change => "https://events.pagerduty.com/v2/change/enqueue",
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
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
            routing_key: Some("integration_key".to_owned()),
            payload: ChangePayload {
                summary: "Hello".to_owned(),
                source: Some("hostname".to_owned()),
                timestamp: Some(Utc.timestamp_millis(2000071804323)),
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
        assert_eq!(cr.unwrap(), "{\"routing_key\":\"integration_key\",\"payload\":{\"summary\":\"Hello\",\"source\":\"hostname\",\"timestamp\":\"2033-05-18T23:30:04.323Z\",\"custom_details\":{\"some_field\":\"Serialize this!\",\"another_field\":34}},\"links\":[{\"href\":\"https://polyverse.com\",\"text\":\"Polyverse homepage\"}]}");

        // With nothing optional
        let c = Change::<()> {
            routing_key: None,
            payload: ChangePayload {
                summary: "Hello".to_owned(),
                source: None,
                timestamp: None,
                custom_details: None,
            },
            links: None,
        };

        let cr = serde_json::to_string(&c);
        assert!(cr.is_ok());
        assert_eq!(cr.unwrap(), "{\"payload\":{\"summary\":\"Hello\"}}");
    }

    #[test]
    fn serialize_alert() {
        // With everything optional
        let a = Alert {
            routing_key: Some("integration_key".to_owned()),
            payload: AlertPayload {
                summary: "Hello".to_owned(),
                source: "hostname".to_owned(),
                timestamp: Some(Utc.timestamp_millis(2000071804323)),
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
            event_action: Action::Trigger,
            client: Some("Zerotect".to_owned()),
            client_url: Some("https://github.com/polyverse/zerotect".to_owned()),
        };

        let ar = serde_json::to_string(&a);
        assert!(ar.is_ok());
        assert_eq!(ar.unwrap(), "{\"routing_key\":\"integration_key\",\"payload\":{\"severity\":\"info\",\"summary\":\"Hello\",\"source\":\"hostname\",\"timestamp\":\"2033-05-18T23:30:04.323Z\",\"component\":\"postgres\",\"group\":\"prod-datapipe\",\"class\":\"deploy\",\"custom_details\":{\"some_field\":\"Serialize this!\",\"another_field\":34}},\"dedup_key\":\"dedupkey1\",\"images\":[{\"src\":\"https://polyverse.com/static/img/SplashPageIMG/polyverse_blue.png\",\"href\":\"https://polyverse.com\",\"alt\":\"The Polyverse Logo\"}],\"links\":[{\"href\":\"https://polyverse.com\",\"text\":\"Polyverse homepage\"}],\"event_action\":\"trigger\",\"client\":\"Zerotect\",\"client_url\":\"https://github.com/polyverse/zerotect\"}");

        // With nothing optional
        let a = Alert::<()> {
            routing_key: None,
            payload: AlertPayload {
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
            event_action: Action::Resolve,
            client: None,
            client_url: None,
        };

        let ar = serde_json::to_string(&a);
        assert!(ar.is_ok());
        assert_eq!(ar.unwrap(), "{\"payload\":{\"severity\":\"info\",\"summary\":\"Hello\",\"source\":\"hostname\"},\"event_action\":\"resolve\"}");
    }
}
