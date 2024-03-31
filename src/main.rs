use reqwest;
use embryo::{EmbryoList, EmPair};
use textwrap::wrap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

const CONF_FILE : &str = "emergence.conf";

fn read_emergence_conf() -> Option<HashMap<String, HashMap<String, String>>> {
    let config_path = match dirs::config_dir() {
        Some(mut path) => {
            path.push("emergence");
            path.push(CONF_FILE);
            path
        }
        None => return None, 
    };

    if !config_path.exists() {
        // Try %APPDATA% on Windows
        let mut appdata_path = PathBuf::new();
        if let Some(appdata) = std::env::var_os("APPDATA") {
            appdata_path.push(appdata);
            appdata_path.push("emergence");
            appdata_path.push(CONF_FILE);

            if appdata_path.exists() {
                return Some(read_file(&appdata_path));
            }
        }

        return None;
    }

    Some(read_file(&config_path))
}

fn read_file(path: &Path) -> HashMap<String, HashMap<String, String>> {
    let file = File::open(path).expect(&format!("Can't open path : {:?}", path));
    let reader = io::BufReader::new(file);
    let mut map = HashMap::new();
    let mut current_section = String::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            let line = line.trim();
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                map.insert(current_section.clone(), HashMap::new());
            } else {
                let parts: Vec<&str> = line.splitn(2, '=').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let value = parts[1].trim_matches('"').to_string();
                    if let Some(section_map) = map.get_mut(&current_section) {
                        section_map.insert(parts[0].to_string(), value);
                    }
                }
            }
        }
    }

    map
}

#[tokio::main]
async fn main() {
    let mut server_url = "http://localhost:8080";
    let config_map = match read_emergence_conf() {
        Some(map) => map,
        None => HashMap::new(),
    };
    if let Some(em_disco) = config_map.get("em_disco") {
        if let Some(url) = em_disco.get("server_url") {
            server_url = url;
        }
    }

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

