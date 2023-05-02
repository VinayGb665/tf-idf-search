use serde_json::json;
use std::{collections::HashMap, fs::File, io::Result};
use tiny_http::{Header, Method, Request, Response, Server};

use crate::{models::SearchResponse, tf_relevance};

type IndexDoc = HashMap<String, HashMap<String, usize>>;

pub fn router(mut request: Request, document_map: &IndexDoc) -> Result<()> {
    let reqpath = "/".to_string()
        + &request
            .url()
            .split("/")
            .skip(1)
            .collect::<Vec<_>>()
            .join("/");

    println!("Request path was {reqpath:?}");
    match reqpath.as_str() {
        "/" => {
            println!("Rendering UIsss");
            let response = Response::from_file(File::open("index.html")?)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
            request.respond(response)?
        }
        "/search" => match request.method() {
            Method::Post => {
                let mut body_content = String::new();
                request.as_reader().read_to_string(&mut body_content)?;
                println!("Searching for {body_content}");
                let results = tf_relevance(&body_content, document_map);
                let json_response = json!(SearchResponse {
                    len: results.keys().len(),
                    results: results,
                });
                let response = Response::from_data(json_response.to_string().into_bytes())
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
                    );
                request.respond(response)?;
            }
            _ => {
                let response = Response::from_string("Not Found").with_status_code(404);
                request.respond(response)?
            }
        },
        _ => {
            let response = Response::from_string("Not Found").with_status_code(404);
            request.respond(response)?
        }
    }
    Ok(())
}
pub fn start_server(index: &IndexDoc) {
    let server = Server::http("0.0.0.0:3663").unwrap();
    loop {
        let request = match server.recv() {
            Ok(request) => {
                router(request, index);
            }
            Err(e) => {
                eprintln!("Error when reading message : {e}");
            }
        };
    }
}
