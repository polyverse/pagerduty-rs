use crate::private_types::*;
use crate::types::*;

use reqwest::header::{
    HeaderMap, HeaderValue, InvalidHeaderValue, CONTENT_ENCODING, CONTENT_TYPE, USER_AGENT,
};
use reqwest::Client;
use serde::Serialize;
use std::convert::From;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

const CONTENT_ENCODING_IDENTITY: &str = "identity";
const CONTENT_TYPE_JSON: &str = "application/json";

#[derive(Debug)]
pub enum EventsV2Error {
    ReqwestError(reqwest::Error),
    InvalidHeaderValue(InvalidHeaderValue),

    //https://developer.pagerduty.com/docs/events-api-v2/overview/#api-response-codes--retry-logic
    HttpNotAccepted(u16), // NOT 4xx, 5xx or 200 (we expect 202). Contains HTTP response code.
    HttpError(u16),       // A legit error (4xx or 5xx). Contains HTTP response code.
}

impl Error for EventsV2Error {}
impl Display for EventsV2Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::ReqwestError(e) => write!(f, "RequestError: {}", e),
            Self::InvalidHeaderValue(e) => write!(f, "InvalidHeaderValue: {}", e),
            Self::HttpNotAccepted(e) => write!(f, "HttpNotAccepted: {}", e),
            Self::HttpError(e) => write!(f, "HttpError: {}", e),
        }
    }
}
impl From<reqwest::Error> for EventsV2Error {
    fn from(err: reqwest::Error) -> Self {
        Self::ReqwestError(err)
    }
}
impl From<InvalidHeaderValue> for EventsV2Error {
    fn from(err: InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(err)
    }
}

pub type EventsV2Result = Result<(), EventsV2Error>;

/// The main PagerDuty Events V2 API
pub struct EventsV2 {
    /// The integration/routing key for a generated PagerDuty service
    integration_key: String,
    client: Client,
}

impl EventsV2 {
    pub fn new(
        integration_key: String,
        user_agent: Option<String>,
    ) -> Result<EventsV2, EventsV2Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(CONTENT_TYPE_JSON)?);
        headers.insert(
            CONTENT_ENCODING,
            HeaderValue::from_str(CONTENT_ENCODING_IDENTITY)?,
        );
        if let Some(ua) = user_agent {
            headers.insert(USER_AGENT, HeaderValue::from_str(ua.as_str())?);
        }

        let client = Client::builder().default_headers(headers).build()?;

        Ok(EventsV2 {
            integration_key,
            client,
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
        let res = self.client.post(url).json(&content).send().await?;

        match res.status().as_u16() {
            202 => Ok(()),
            e if e < 400 => Err(EventsV2Error::HttpNotAccepted(e)),
            e => Err(EventsV2Error::HttpError(e)),
        }
    }
}
