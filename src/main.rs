use reqwest;
use embryo::{EmbryoList, EmPair};
use textwrap::wrap;

#[tokio::main]
async fn main() {
    let server_url = "http://localhost:8080";

    let client = reqwest::Client::new();

    let args: Vec<String> = std::env::args().collect();
    let query = args[1..].join(" ");

    let response = client
        .post(&format!("{}/query", server_url))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(query)
        .send()
        .await
        .expect("Failed to send HTTP request");

    if response.status().is_success() {
        let body = response.text().await.expect("Failed to get response body");

        let filter_response: Result<EmbryoList, _> = serde_json::from_str(&body);

        match filter_response {
            Ok(filter_response) => {
                for embryo in filter_response.embryo_list {
                    let mut url = String::new();
                    let mut resume = String::new();
                    for pair in &embryo.properties {
                        match pair.name.as_str() {
                            "url"  => {
                                url=pair.value.clone();
                            },
                            "resume" => {
                                resume=pair.value.clone();
                            }
                            _ => { }
                        }
                    }
                    let term_width = match term_size::dimensions() {
                        Some((w, _)) => w as usize,
                        None => 80,
                    };

                    let wrapped_resume = wrap(&resume, term_width - 1)
                        .iter()
                        .map(|line| format!("\t{}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    println!("{}\n{}", url, wrapped_resume);
                }
            }
            Err(_) => {
                let uri = body.trim_matches('"').to_owned();
                let em_pair = EmPair {
                    name: "url".to_owned(),
                    value: uri,
                };
                println!("Single EmPair: {:?}", em_pair);
            }
        }
    } else {
        println!("Failed to get a successful response. Status code: {}", response.status());
    }
}

