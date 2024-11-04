pub async fn update_a_record(
    root_url: &str,
    api_key: &str,
    zone: &str,
    name: &str,
    content: &str,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/servers/localhost/zones/{}.", root_url, zone);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-API-Key", api_key.parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let body = format!(
        r#"
        {{
            "rrsets": [
                {{
                    "name": "{name}.",
                    "type": "A",
                    "ttl": 60,
                    "changetype": "REPLACE",
                    "records": [
                        {{
                            "content": "{content}",
                            "disabled": false
                        }}
                    ]
                }}
            ]
        }}
        "#
    );

    tracing::info!("PATCH: {}", url);

    tracing::debug!("{}", body);

    let res = client.patch(url).headers(headers).body(body).send().await?;
    let status = res.status();
    let text = res.text().await?;

    tracing::debug!("Response: {text:?}",);

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Failed to update A record: status is {status} and text is {text}"
        ));
    }

    Ok(())
}
