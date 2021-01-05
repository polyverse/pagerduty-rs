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

impl<T> SendableChange<T>
where
    T: Serialize,
{
    pub fn from_change(change: Change<T>, integration_key: String) -> Self {
        SendableChange::<T> {
            routing_key: integration_key,
            links: change.links,
            payload: change.payload,
        }
    }
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

impl<T> SendableAlertTrigger<T>
where
    T: Serialize,
{
    pub fn from_alert_trigger(alert_trigger: AlertTrigger<T>, integration_key: String) -> Self {
        SendableAlertTrigger::<T> {
            routing_key: integration_key,
            event_action: Action::Trigger,
            dedup_key: alert_trigger.dedup_key,
            images: alert_trigger.images,
            links: alert_trigger.links,
            payload: alert_trigger.payload,
            client: alert_trigger.client,
            client_url: alert_trigger.client_url,
        }
    }
}

#[derive(Serialize)]
pub struct SendableAlertFollowup {
    pub routing_key: String,
    pub dedup_key: String,
    pub event_action: Action,
}

impl SendableAlertFollowup {
    pub fn new(dedup_key: String, action: Action, integration_key: String) -> Self {
        SendableAlertFollowup {
            routing_key: integration_key,
            event_action: action,
            dedup_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json;
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
