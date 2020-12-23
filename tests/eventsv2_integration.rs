use pagerduty_rs::*;
use rand::{thread_rng, Rng};
use serde::Serialize;
use time::OffsetDateTime;

/// Set with some integration key value before running tests that post directly to service
const INTEGRATION_KEY: Option<&str> = None;

#[derive(Serialize)]
pub struct SerializableTest {
    some_field: String,
    another_field: usize,
}

#[test]
fn post_change_maximum() {
    if let Some(ik) = INTEGRATION_KEY {
        let e = Event::Change(Change {
            payload: ChangePayload {
                summary: "Change Event 1 maximum fields".to_owned(),
                source: Some("hostname".to_owned()),
                timestamp: OffsetDateTime::now_utc(),
                custom_details: Some(SerializableTest {
                    some_field: "Serialize this!".to_owned(),
                    another_field: 34,
                }),
            },
            links: Some(vec![Link {
                href: "https://polyverse.com".to_owned(),
                text: Some("Polyverse homepage".to_owned()),
            }]),
        });

        let ev2 = EventsV2::new(ik.to_owned(), Some("pagerduty-rs test".to_owned())).unwrap();

        let result = ev2.event(e);
        assert!(result.is_ok());
    }
}

#[test]
fn post_change_minimum() {
    if let Some(ik) = INTEGRATION_KEY {
        // With nothing optional
        let e = Event::Change(Change::<()> {
            payload: ChangePayload {
                summary: "Change event 2 minimum fields (routing key in api client)".to_owned(),
                timestamp: OffsetDateTime::now_utc(),
                source: None,
                custom_details: None,
            },
            links: None,
        });

        let ev2 = EventsV2::new(ik.to_owned(), Some("pagerduty-rs test".to_owned())).unwrap();

        let result = ev2.event(e);
        assert!(result.is_ok());
    }
}

#[test]
fn post_alert_maximum_trigger_acknowledge_resolve() {
    if let Some(ik) = INTEGRATION_KEY {
        let mut rng = thread_rng();
        let dedup_key = format!("TestDeDupKey{}", rng.gen_range(0..100));

        let ev2 = EventsV2::new(ik.to_owned(), Some("pagerduty-rs test".to_owned())).unwrap();
        // With everything
        let e = Event::AlertTrigger(AlertTrigger {
            payload: AlertTriggerPayload {
                summary: "Test Alert 1 Maximum fields".to_owned(),
                source: "hostname".to_owned(),
                timestamp: Some(OffsetDateTime::now_utc()),
                severity: Severity::Info,
                component: Some("postgres".to_owned()),
                group: Some("prod-datapipe".to_owned()),
                class: Some("deploy".to_owned()),
                custom_details: Some(SerializableTest {
                    some_field: "Serialize this!".to_owned(),
                    another_field: 34,
                }),
            },
            dedup_key: Some(dedup_key.clone()),
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
        });

        let result = ev2.event(e);
        assert!(result.is_ok());

        let e = Event::AlertAcknowledge::<()>(AlertAcknowledge {
            dedup_key: dedup_key.clone(),
        });
        let result = ev2.event(e);
        assert!(result.is_ok());

        let e = Event::AlertResolve::<()>(AlertResolve { dedup_key });
        let result = ev2.event(e);
        assert!(result.is_ok());
    }
}

#[test]
fn post_alert_minimum_trigger_acknowledge_resolve() {
    if let Some(ik) = INTEGRATION_KEY {
        let mut rng = thread_rng();
        let dedup_key = format!("TestDeDupKey{}", rng.gen_range(0..100));

        let ev2 = EventsV2::new(ik.to_owned(), None).unwrap();
        // With everything
        let e = Event::AlertTrigger::<()>(AlertTrigger {
            payload: AlertTriggerPayload {
                summary: "Test Alert 1 Minimum fields".to_owned(),
                source: "hostname".to_owned(),
                timestamp: None,
                severity: Severity::Info,
                component: None,
                group: None,
                class: None,
                custom_details: None,
            },
            dedup_key: Some(dedup_key.clone()),
            images: None,
            links: None,
            client: None,
            client_url: None,
        });

        let result = ev2.event(e);
        assert!(result.is_ok());

        let e = Event::AlertAcknowledge::<()>(AlertAcknowledge {
            dedup_key: dedup_key.clone(),
        });
        let result = ev2.event(e);
        assert!(result.is_ok());

        let e = Event::AlertResolve::<()>(AlertResolve { dedup_key });
        let result = ev2.event(e);
        assert!(result.is_ok());
    }
}
