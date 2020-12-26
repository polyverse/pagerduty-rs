
![Build Status](https://github.com/polyverse/pagerduty-rs/workflows/Build%20Status/badge.svg)

# pagerduty-rs

A PagerDuty Events V2 API Client Library in Rust.

## Using the API

```.rust
use pagerduty_rs::eventsv2::*;

// ....

// Create an API client with an Integration Key
let ev2 = EventsV2::new(String::from("IntegrationKey"), Some("Optional pagerduty-rs user agent".to_owned())).unwrap();

// Then send an event (which might be a change, alert trigger/acknowledge/resolve)...
ev2.event(Event::AlertTrigger(AlertTrigger{
    // ...
}));
```
