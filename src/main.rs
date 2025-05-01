use anyhow::Result;
use badgemagic::{
    embedded_graphics::{
        self, geometry::Point, mono_font::MonoTextStyle, pixelcolor::BinaryColor, text::Text,
    },
    protocol::{Mode, PayloadBuffer, Style},
    usb_hid::Device,
};
use calendar3::CalendarHub;
use chrono::{DateTime, Utc};
use config::Config;
use google_calendar3::{
    self as calendar3,
    api::{Event, EventDateTime},
    hyper_rustls::HttpsConnector,
    hyper_util::client::legacy::connect::HttpConnector,
};
use kakasi::IsJapanese;
use rust_translate::translate_to_english;

mod clients;
mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::try_from_env()?;
    let hub = clients::calendar().await?;

    let events = fetch_events(&hub, &config).await?;

    let mut payload = PayloadBuffer::new();

    for mut event in events {
        println!(
            "\nEVENT => {:?}, {:?}, {:?}",
            event.summary, event.start, event.recurring_event_id
        );

        if config.translate_japanese_to_english {
            match kakasi::is_japanese(&event.summary.clone().unwrap_or_default()) {
                IsJapanese::True | IsJapanese::Maybe => {
                    println!("Japanese detected, atttempting to translate");

                    if let Ok(translation) =
                        translate_to_english(&event.summary.clone().unwrap_or_default()).await
                    {
                        println!("Got translation {translation}");
                        event.summary = Some(translation);
                    };
                }
                IsJapanese::False => {}
            }
        }

        let Some(message) = format_event_message(&event).await else {
            println!("Skipping event due to missing fields :: {event:#?}");
            continue;
        };

        println!("Adding message: {message}");
        add_message(&mut payload, &message);
    }

    Device::single()?.write(payload)?;

    Ok(())
}

async fn fetch_events(
    hub: &CalendarHub<HttpsConnector<HttpConnector>>,
    config: &Config,
) -> Result<Vec<Event>> {
    let now: DateTime<Utc> = Utc::now();
    let end: DateTime<Utc> =
        now + std::time::Duration::from_secs(60 * 60 * 24 * config.days_in_advance);

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

async fn format_event_message(event: &Event) -> Option<String> {
    let Event {
        summary: Some(title),
        start: Some(EventDateTime {
            date_time, date, ..
        }),
        ..
    } = event
    else {
        return None;
    };

    let message = if let Some(time) = date_time {
        let formatted_date = {
            let a = time.format("%B %d, %I:%M %p");
            a.to_string()
        };

        format!("{title} -- {}", formatted_date.to_string())
    } else if let Some(date) = date {
        let formatted_date = {
            let a = date.format("%B %d");
            a.to_string()
        };

        format!("{title} -- {}", formatted_date.to_string())
    } else {
        title.to_string()
    };
    return Some(message);
}

fn add_message(payload: &mut PayloadBuffer, message: &str) {
    payload.add_message_drawable(
        Style::default().mode(Mode::Left),
        &Text::new(
            &message,
            Point::new(0, 8),
            MonoTextStyle::new(
                &embedded_graphics::mono_font::iso_8859_1::FONT_6X9,
                BinaryColor::On,
            ),
        ),
    );
}
