use crate::private_types::*;
use crate::types::*;

use hyper::{client::HttpConnector, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde::Serialize;
use std::convert::From;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

const CONTENT_TYPE: &str = "content-type";
const USER_AGENT: &str = "user-agent";
const CONTENT_ENCODING: &str = "content-encoding";

const CONTENT_ENCODING_IDENTITY: &str = "identity";
const CONTENT_TYPE_JSON: &str = "application/json";

#[derive(Debug)]
pub enum EventsV2Error {
    SerdeJsonError(serde_json::Error),
    HyperError(hyper::Error),
    HyperHttpError(hyper::http::Error),
    //https://developer.pagerduty.com/docs/events-api-v2/overview/#api-response-codes--retry-logic
    HttpNotAccepted(u16), // NOT 4xx, 5xx or 200 (we expect 202). Contains HTTP response code.
    HttpError(u16),       // A legit error (4xx or 5xx). Contains HTTP response code.
}

impl Error for EventsV2Error {}
impl Display for EventsV2Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::SerdeJsonError(e) => write!(f, "SerdeJsonError: {}", e),
            Self::HyperHttpError(e) => write!(f, "HyperHttpError: {}", e),
            Self::HyperError(e) => write!(f, "HyperError: {}", e),
            Self::HttpNotAccepted(e) => write!(f, "HttpNotAccepted: {}", e),
            Self::HttpError(e) => write!(f, "HttpError: {}", e),
        }
    }
}
impl From<serde_json::Error> for EventsV2Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJsonError(err)
    }
}
impl From<hyper::http::Error> for EventsV2Error {
    fn from(err: hyper::http::Error) -> Self {
        Self::HyperHttpError(err)
    }
}
impl From<hyper::Error> for EventsV2Error {
    fn from(err: hyper::Error) -> Self {
        Self::HyperError(err)
    }
}

pub type EventsV2Result = Result<(), EventsV2Error>;

/// The main PagerDuty Events V2 API
pub struct EventsV2 {
    /// The integration/routing key for a generated PagerDuty service
    integration_key: String,
    client: Client<HttpsConnector<HttpConnector>>,
    user_agent: Option<String>,
}

impl EventsV2 {
    pub fn new(
        integration_key: String,
        user_agent: Option<String>,
    ) -> Result<EventsV2, EventsV2Error> {
        let https = HttpsConnector::new();

        Ok(EventsV2 {
            integration_key,
            user_agent,
            client: Client::builder().build::<_, hyper::Body>(https),
        })
    }

    pub async fn event<T: Serialize>(&self, event: Event<T>) -> EventsV2Result {
        match event {
            Event::Change(c) => self.change(c).await,
            Event::AlertTrigger(at) => self.alert_trigger(at).await,
            Event::AlertAcknowledge(aa) => {
                self.alert_followup(aa.dedup_key, Action::Acknowledge).await
            }
            Event::AlertResolve(ar) => self.alert_followup(ar.dedup_key, Action::Resolve).await,
        }
    }

    async fn change<T: Serialize>(&self, change: Change<T>) -> EventsV2Result {
        let sendable_change = SendableChange::from_change(change, self.integration_key.clone());

        self.do_post(
            "https://events.pagerduty.com/v2/change/enqueue",
            sendable_change,
        )
        .await
    }

    async fn alert_trigger<T: Serialize>(&self, alert_trigger: AlertTrigger<T>) -> EventsV2Result {
        let sendable_alert_trigger =
            SendableAlertTrigger::from_alert_trigger(alert_trigger, self.integration_key.clone());

        self.do_post(
            "https://events.pagerduty.com/v2/enqueue",
            sendable_alert_trigger,
        )
        .await
    }

    async fn alert_followup(&self, dedup_key: String, action: Action) -> EventsV2Result {
        let sendable_alert_followup =
            SendableAlertFollowup::new(dedup_key, action, self.integration_key.clone());

        self.do_post(
            "https://events.pagerduty.com/v2/enqueue",
            sendable_alert_followup,
        )
        .await
    }

    // Make this part Async in the future
    async fn do_post<T: Serialize>(&self, url: &str, content: T) -> EventsV2Result {
        let jsonstr = serde_json::to_string(&content)?;

        let mut reqbldr = Request::builder()
            .method("POST")
            .uri(url)
            .header(CONTENT_TYPE, CONTENT_TYPE_JSON)
            .header(CONTENT_ENCODING, CONTENT_ENCODING_IDENTITY);

        reqbldr = if let Some(ua) = &self.user_agent {
            reqbldr.header(USER_AGENT, ua)
        } else {
            reqbldr
        };

        let req = reqbldr.body(Body::from(jsonstr))?;

        let res = self.client.request(req).await?;

        match res.status().as_u16() {
            202 => Ok(()),
            e if e < 400 => Err(EventsV2Error::HttpNotAccepted(e)),
            e => Err(EventsV2Error::HttpError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use time::OffsetDateTime;

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
