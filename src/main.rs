use anyhow::Result;
use calendar3::{hyper_rustls, hyper_util, yup_oauth2, CalendarHub};
use chrono::{DateTime, Utc};
use google_calendar3::{
    self as calendar3, hyper_rustls::HttpsConnector,
    hyper_util::client::legacy::connect::HttpConnector,
};

#[tokio::main]
async fn main() -> Result<()> {
    let hub = create_client().await?;

    let now: DateTime<Utc> = Utc::now();
    let end: DateTime<Utc> = now + std::time::Duration::from_secs(60 * 60 * 24 * 14);

    let result = hub
        .events()
        .list("")
        .time_min(now)
        .time_max(end)
        .single_events(true)
        .doit()
        .await?;

    for event in result.1.items.unwrap_or_default() {
        println!(
            "EVENT => {:?}, {:?}, {:?}",
            event.summary, event.start, event.recurring_event_id
        );
    }

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
