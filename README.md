# DEPRECATION NOTICE

Please note that this repository has been deprecated and is no longer actively maintained by Polyverse Corporation.  It may be removed in the future, but for now remains public for the benefit of any users.

Importantly, as the repository has not been maintained, it may contain unpatched security issues and other critical issues.  Use at your own risk.

While it is not maintained, we would graciously consider any pull requests in accordance with our Individual Contributor License Agreement.  https://github.com/polyverse/contributor-license-agreement

For any other issues, please feel free to contact info@polyverse.com

---

![Build Status](https://github.com/polyverse/pagerduty-rs/workflows/Build%20Status/badge.svg)

# pagerduty-rs

A PagerDuty Events V2 API Client Library in Rust.

## Using the API

Complete API examples are provided as [integration tests](./tests).

With feature `sync`:

```.rust
use pagerduty_rs::eventsv2sync::*;
use pagerduty_rs::types::*;

// ....

// Create an API client with an Integration Key
let ev2 = EventsV2::new(String::from("IntegrationKey"), Some("Optional pagerduty-rs user agent".to_owned())).unwrap();

// Then send an event (which might be a change, alert trigger/acknowledge/resolve)...
ev2.event(Event::AlertTrigger(AlertTrigger{
    // ...
}));
```

With feature `async`:

```.rust
use pagerduty_rs::eventsv2async::*;
use pagerduty_rs::types::*;

// ....

// Create an API client with an Integration Key
let ev2 = EventsV2::new(String::from("IntegrationKey"), Some("Optional pagerduty-rs user agent".to_owned())).unwrap();

// Then send an event (which might be a change, alert trigger/acknowledge/resolve)...
ev2.event(Event::AlertTrigger(AlertTrigger{
    // ...
})).await;
```

