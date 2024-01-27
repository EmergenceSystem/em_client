use reqwest;
use embryo::{EmbryoList, EmPair};

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
                    for pair in &embryo.properties {
                        if pair.name == "url" {
                            let em_pair = EmPair {
                                name: pair.name.clone(),
                                value: pair.value.clone(),
                            };
                            println!("{:?}", em_pair);
                        }
                    }
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

