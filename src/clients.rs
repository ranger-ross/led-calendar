use anyhow::Result;
use calendar3::{hyper_rustls, hyper_util, yup_oauth2, CalendarHub};
use google_calendar3::{
    self as calendar3, hyper_rustls::HttpsConnector,
    hyper_util::client::legacy::connect::HttpConnector,
};

pub async fn calendar() -> Result<CalendarHub<HttpsConnector<HttpConnector>>> {
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
