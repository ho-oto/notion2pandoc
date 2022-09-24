use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json;

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

pub async fn embed_html(url: &str) -> Option<String> {
    #[derive(Debug, Deserialize)]
    struct Oembed {
        html: String,
    }

    if let Some(oembed_endpoint) = oembed_by_providers_json(url) {
        let params = vec![("url", url), ("format", "json")];
        let resp = Client::new()
            .get(oembed_endpoint.replace(r"{format}", "json"))
            .query(&params)
            .send()
            .await;
        if let Ok(resp) = resp {
            let html = resp.json::<Oembed>().await;
            if let Ok(embed) = html {
                return Some(embed.html);
            }
        }
    }

    if let Some(page) = Client::new().get(url).send().await.ok() {
        let mut oembed_url = None;
        let html = page.text().await.ok();
        if let Some(html) = html {
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

#[tokio::test]
async fn test_oembed() {
    println!(
        "{}",
        embed_html("https://follower-anone.hatenablog.jp/entry/2022/09/11/214308")
            .await
            .unwrap_or("no".to_string())
    );
}

//pub fn embed_html(url: String) -> Option<String> {
//    // support gist, github, dropbox, google drive
//
//    // use oembed providers
//
//    // search by link tag
//}
//
