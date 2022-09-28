use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json;

fn embed_tag_exceptional(url: &str) -> Option<String> {
    if Regex::new(r"https://gist\.github\.com/*")
        .unwrap()
        .is_match(url)
    {
        Some(format!(r#"<script src="{}.js"></script>"#, url))
    } else {
        None
    }
}

fn oembed_by_providers_json(url: &str) -> Option<String> {
    let json = include_str!("../embed/providers.json");
    #[derive(Debug, Deserialize)]
    struct Provider {
        endpoints: Vec<Endpoint>,
    }
    #[derive(Debug, Deserialize)]
    struct Endpoint {
        schemes: Option<Vec<String>>,
        url: String,
    }
    let providers: Vec<Provider> = serde_json::from_str(json).unwrap();

    for provider in providers {
        for endpoint in provider.endpoints {
            if let Some(schemes) = endpoint.schemes {
                for scheme in schemes {
                    let mut quoted = String::new();
                    quoted.reserve(scheme.len());
                    for c in scheme.chars() {
                        match c {
                            '\\' | '.' | '+' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}'
                            | '^' | '$' | '#' | '&' | '-' | '~' => {
                                quoted.push('\\');
                                quoted.push(c)
                            }
                            '*' => {
                                quoted.push('.');
                                quoted.push('+')
                            }
                            _ => quoted.push(c),
                        }
                    }
                    if Regex::new(&quoted).unwrap().is_match(url) {
                        return Some(endpoint.url);
                    }
                }
            }
        }
    }
    None
}

pub async fn embed_tag(url: &str) -> Option<String> {
    if let Some(tag) = embed_tag_exceptional(url) {
        return Some(tag);
    }

    #[derive(Debug, Deserialize)]
    struct Oembed {
        html: String,
    }

    // get embed tag by oEmbed API listed in providers.json
    // https://oembed.com/providers.json
    if let Some(oembed_endpoint) = oembed_by_providers_json(url) {
        let resp = Client::new()
            .get(oembed_endpoint.replace(r"{format}", "json"))
            .query(&vec![("url", url), ("format", "json"), ("maxwidth", "720")])
            .send()
            .await;
        if let Ok(resp) = resp {
            if let Ok(embed) = resp.json::<Oembed>().await {
                return Some(embed.html);
            }
        }
    }

    // try to use oEmbed discovery mechanism
    // https://oembed.com/#section4
    if let Ok(page) = Client::new().get(url).send().await {
        let mut oembed_url = None;
        if let Ok(html) = page.text().await {
            let html = Html::parse_document(&html);
            let selector =
                Selector::parse(r#"head > link[type="application/json+oembed"]"#).unwrap();
            for elm in html.select(&selector) {
                oembed_url = elm.value().attr("href").map(|s| s.to_string());
            }
        }
        if let Some(oembed_url) = oembed_url {
            let resp = Client::new().get(oembed_url).send().await;
            if let Ok(resp) = resp {
                if let Ok(embed) = resp.json::<Oembed>().await {
                    return Some(embed.html);
                }
            }
        }
    }

    None
}
