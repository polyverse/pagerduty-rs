use crate::types::*;

use serde::{Serialize, Serializer};
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
