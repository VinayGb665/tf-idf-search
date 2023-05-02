use glob::glob;
use lexer::Lexer;
use serde::{Deserialize, Serialize};
use serde_json::json;
use server::start_server;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use xml::reader::Error;
use xml::reader::{EventReader, XmlEvent};
use xml::writer::events;
pub mod lexer;
pub mod models;
pub mod server;
const INDEX_FILE: &str = "index.json";
#[derive(Serialize, Deserialize)]
struct TfTable(HashMap<String, HashMap<String, usize>>);
fn read_xml_and_parse(file_path: &str) -> Result<(String), Error> {
    let mut parsed_content = String::new();
    let xml_content = fs::read_to_string(file_path)?;

    let xml_reader = EventReader::from_str(&xml_content);
    for event in xml_reader.into_iter() {
        let event = event?;
        if let XmlEvent::Characters(mut text) = event {
            parsed_content.push_str(&mut text);
            parsed_content.push_str(" ");
        }
    }
    Ok(parsed_content)
}

fn process_file(file_path: &str) -> HashMap<String, usize> {
    println!("Indexing file {file_path}");
    // let xml_file = File::open(file_path).unwrap();
    let parsed_content = read_xml_and_parse(file_path);
    if (parsed_content.is_err()) {
        return HashMap::new();
    }

    let content_chars = parsed_content
        .expect("I dont know how i ended up here help me :(")
        .to_ascii_lowercase()
        .chars()
        .collect::<Vec<_>>();
    let lexer = Lexer::new(&content_chars);
    let mut frequencyMap: HashMap<String, usize> = HashMap::new();
    for token in lexer.into_iter() {
        // println!(
        //     "Token is {token:?}",
        //     token = token.iter().collect::<String>()
        // );
        let mut entry = frequencyMap.entry(token).or_insert(0 as usize);
        *entry += 1;
    }
    // println!("Frequency map is {fMap:?}", fMap = frequencyMap);
    frequencyMap
}

fn index_corpus(folder_path: &str) -> io::Result<HashMap<String, HashMap<String, usize>>> {
    let mut masterFrequencyMap = HashMap::new();
    let mut document_map = HashMap::new();
    let mut document_count_map = HashMap::new();
    for entry in glob(folder_path).expect("Could not see glob glob") {
        let dirEntry = entry.unwrap_or_else(|e| {
            eprintln!("Error while reading directory : ${e:?}");
            exit(1)
        });
        if (dirEntry.is_dir()) {
            continue;
        }
        let frequencyMap = match dirEntry.to_str() {
            Some(x) => process_file(x),
            None => HashMap::new(),
        };

        for token in frequencyMap.clone() {
            let entry = masterFrequencyMap
                .entry(token.0.clone())
                .or_insert(0 as usize);
            *entry += token.1;

            let document_count_entry = document_count_map.entry(token.0).or_insert(1);
            *document_count_entry += 1;
        }
        document_map.insert(dirEntry.to_str().unwrap().to_string(), frequencyMap);
    }

    Ok(document_map)
}

fn tf(term: &str, tf_table: &HashMap<String, usize>) -> f32 {
    let mut total_count = 0;
    for (_, count) in tf_table.iter() {
        total_count += count;
    }

    let freq = tf_table.get(term).cloned().unwrap_or(0);
    freq as f32 / total_count as f32
}

fn idf(idf_map: &HashMap<String, i32>, total_docs: usize) -> HashMap<&String, f32> {
    let mut idf_scores = HashMap::new();

    for (token, tf) in idf_map.iter() {
        let score = total_docs as f32 / (1 + tf) as f32;
        idf_scores.insert(token, f32::log10(score));
    }
    idf_scores
}

fn tf_relevance(
    query: &str,
    tf_table: &HashMap<String, HashMap<String, usize>>,
) -> HashMap<String, f32> {
    let mut relevances = Vec::new();
    let mut occurances = HashMap::new();
    let mut tf_master_table = HashMap::new();
    println!(
        "Total number of docs are {doc_len} ",
        doc_len = tf_table.keys().len()
    );
    for (filename, table) in tf_table.iter() {
        let mut relevance: f32 = 0.;
        let mut tf_file = HashMap::new();
        let content = query
            .to_ascii_lowercase()
            .chars()
            .into_iter()
            .collect::<Vec<_>>();
        let lexer = Lexer::new(&content);
        // Create our query vector with the lexer
        for token in lexer {
            let score = tf(token.clone().as_str(), table);
            tf_file.insert(token.clone(), score);
            relevance += score;
            if score > 0.0 {
                let entry = occurances.entry(token).or_insert(0);
                *entry += 1;
            }
        }
        tf_master_table.insert(filename, tf_file);

        relevances.push((filename, relevance));
    }

    let idf_scores = idf(&occurances, tf_table.keys().len());
    let mut tf_idf_scores = Vec::new();
    for (filename, tf_file_table) in tf_master_table.iter() {
        let mut query_tf_idf_score = 0.0;
        for (token, score) in tf_file_table.iter() {
            let idf_score = idf_scores.get(token).cloned().unwrap_or(0.0);
            query_tf_idf_score += idf_score * score;
        }
        tf_idf_scores.push((filename, query_tf_idf_score));
    }

    tf_idf_scores.sort_by(|(_, rank1), (_, rank2)| rank1.partial_cmp(rank2).unwrap());
    tf_idf_scores.reverse();
    println!(
        "Top 10 relevant documents were: {first:?} {last:?}",
        first = tf_idf_scores.first(),
        last = tf_idf_scores.last()
    );
    let mut top_n = 10;

    let mut result = HashMap::new();
    for (f, score) in tf_idf_scores.iter() {
        top_n -= 1;
        result.insert(f.deref().deref().clone(), score.clone());
        if top_n == 0 || *score <= 0. {
            break;
        }
    }
    result
}

fn usage() {
    println!("USAGE: cargo run <command>");
    println!("Commands:");
    println!("  index \t\t\t  Run index on the directory");
    println!("  search <query> \t\t\t Search the query with tf-idf ")
}

fn index() -> io::Result<()> {
    let dir_path = "/Users/vinay/Documents/docs.gl/**/*.xhtml";
    let document_map = index_corpus(dir_path)?;
    let json_contents = json!(document_map).to_string();
    fs::write(INDEX_FILE, json_contents)?;
    Ok(())
}

fn load_index(force: bool) -> io::Result<HashMap<String, HashMap<String, usize>>> {
    let reader = fs::read(INDEX_FILE).unwrap();
    let document_map: TfTable =
        serde_json::from_str(&String::from_utf8(reader).unwrap().as_str()).unwrap();
    Ok(document_map.0)
}

fn main() -> io::Result<()> {
    let index_file = "index.json";
    let mut args = env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        let cmd = args[1].clone();
        match cmd.as_str() {
            "index" => {
                index()?;
                exit(0)
            }
            "search" => {
                if (args.len() <= 2) {
                    usage();
                    exit(1);
                } else {
                    let mut query = String::new();
                    for query_token in args.iter().skip(2) {
                        query.push_str(" ");
                        query.push_str(query_token);
                    }
                    println!("Searching {query}");

                    let document_map = load_index(false)?;
                    tf_relevance(&query.as_str(), &document_map);
                }
            }
            "serve" => {
                let index = load_index(true)?;
                println!("Starting server");
                start_server(&index);
            }

            _ => {
                eprintln!("Invalid command");
                usage()
            }
        }
    } else {
        usage();
        exit(1);
    }

    Ok(())
}
