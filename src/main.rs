use anyhow::Result;
use badgemagic::{
    embedded_graphics::{
        self, geometry::Point, mono_font::MonoTextStyle, pixelcolor::BinaryColor, text::Text,
    },
    protocol::{Mode, PayloadBuffer, Style},
    usb_hid::Device,
};
use calendar3::{hyper_rustls, hyper_util, yup_oauth2, CalendarHub};
use chrono::{DateTime, Utc};
use config::Config;
use google_calendar3::{
    self as calendar3,
    api::{Event, EventDateTime},
    hyper_rustls::HttpsConnector,
    hyper_util::client::legacy::connect::HttpConnector,
};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::try_from_env()?;
    let hub = create_client().await?;

    let events = fetch_events(&hub, &config).await?;

    let mut payload = PayloadBuffer::new();

    for event in events {
        println!(
            "EVENT => {:?}, {:?}, {:?}",
            event.summary, event.start, event.recurring_event_id
        );

        let Some(message) = format_event_message(event) else {
            println!("Skipping event due to missing fields");
            continue;
        };

        add_message(&mut payload, &message);
    }

    Device::single()?.write(payload)?;

    Ok(())
}

async fn create_client() -> Result<CalendarHub<HttpsConnector<HttpConnector>>> {
    let secret = yup_oauth2::read_application_secret("sa.json").await?;
    let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await?;

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()?
                .https_or_http()
                .enable_http1()
                .build(),
        );
    let hub = CalendarHub::new(client, auth);

    return Ok(hub);
}

async fn fetch_events(
    hub: &CalendarHub<HttpsConnector<HttpConnector>>,
    config: &Config,
) -> Result<Vec<Event>> {
    let now: DateTime<Utc> = Utc::now();
    let end: DateTime<Utc> = now + std::time::Duration::from_secs(60 * 60 * 24 * 14);

    let mut events = vec![];

    for calendar_id in &config.calendar_ids {
        let result = hub
            .events()
            .list(calendar_id)
            .time_min(now)
            .time_max(end)
            .single_events(true)
            .doit()
            .await?;

        events.extend(result.1.items.unwrap_or_default());
    }

    return Ok(events);
}

fn format_event_message(event: Event) -> Option<String> {
    let Event {
        summary: Some(title),
        start: Some(EventDateTime {
            date_time: Some(time),
            ..
        }),
        ..
    } = event
    else {
        return None;
    };

    let now = Utc::now();
    let duration = time.signed_duration_since(now).to_std().ok()?;
    let formatted_date = {
        let fd = humantime::format_duration(duration);
        let d = fd.to_string();
        // trim off everything less than hrs
        // TODO: handle the case < 1hr remaining
        if let Some((prefix, _)) = d.split_once("h") {
            format!("{}h", prefix)
        } else {
            d
        }
    };

    let message = format!("{title} ({})", formatted_date.to_string());

    return Some(message);
}

fn add_message(payload: &mut PayloadBuffer, message: &str) {
    payload.add_message_drawable(
        Style::default().mode(Mode::Left),
        &Text::new(
            &message,
            Point::new(0, 7),
            MonoTextStyle::new(
                &embedded_graphics::mono_font::iso_8859_1::FONT_4X6,
                BinaryColor::On,
            ),
        ),
    );
}
