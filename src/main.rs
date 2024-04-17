use reqwest;
use textwrap::wrap;
use embryo::EmbryoList;

#[tokio::main]
async fn main() {
    let server_url = embryo::get_em_disco_url();

    let client = reqwest::Client::new();
    let args: Vec<String> = std::env::args().collect();
    let query = args[1..].join(" ");

    let response = match client
        .post(&format!("{}/query", server_url))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(query)
        .send()
        .await {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Failed to send HTTP request to {} : \n\t{}", server_url, err);
                return;
            }
        };

    if response.status().is_success() {
        let body = response.text().await.expect("Failed to get response body");

        let filter_response: Result<EmbryoList, _> = serde_json::from_str(&body);

        match filter_response {
            Ok(filter_response) => {
                for embryo in filter_response.embryo_list {
                    let mut url = String::new();
                    let mut resume = String::new();
                    for (name, value) in &embryo.properties {
                        match name.as_str() {
                            "url"  => {
                                url=value.clone();
                            },
                            "resume" => {
                                resume=value.clone();
                            }
                            _ => { }
                        }
                    }
                    let term_width = match term_size::dimensions() {
                        Some((w, _)) => w as usize - 10,
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
                println!("{}", uri);
            }
        }
    } else {
        println!("Failed to get a successful response. Status code: {}", response.status());
    }
}

