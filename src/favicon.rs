use reqwest::Client;
use url::Url;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FaviconResponse {
    pub url: String,
    pub host: String,
    pub status: u16,
    #[serde(rename = "statusText")]
    pub status_text: String,
    pub icons: Vec<FaviconIcon>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FaviconIcon {
    pub sizes: String,
    pub href: String,
}

pub async fn download_favicon(url: &str) -> anyhow::Result<Vec<String>> {
    let mut favicons = Vec::new();

    let url = Url::parse(url)?;

    if let Some(domain) = url.domain() {
        let request = Client::new()
            .get(format!(
                "https://www.faviconextractor.com/api/favicon/{domain}"
            ))
            .send()
            .await?;

        let response: FaviconResponse = request.json().await?;

        if response.status == 200 {
            response
                .icons
                .iter()
                .for_each(|icon| favicons.push(icon.href.clone()));
        }
    }

    Ok(favicons)
}
