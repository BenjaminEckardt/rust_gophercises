use std::{
    io::{prelude::*},
    net::{TcpListener, TcpStream},
};
use std::collections::HashMap;
use std::io::BufReader;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let route_mappings = read_mapping();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, &route_mappings);
    }
}

fn handle_connection(mut stream: TcpStream, route_mappings: &HashMap<String, String>) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    match http_request.get(0) {
        Some(first_line) => {
            let path = *first_line.split(" ").collect::<Vec<_>>().get(1).unwrap();
            println!("incoming requests for path {}", path);
            let default = String::from("https://beneck.de");
            let target_url = route_mappings.get(path).unwrap_or(&default);
            stream.write_all(format!("HTTP/1.1 302 Found\nLocation: {}\n\n", target_url).as_bytes()).unwrap()
        },
        None => {
            stream.write_all("HTTP/1.1 400 Bad Request\n\n".as_bytes()).unwrap()
        }
    }
}

fn read_mapping() -> HashMap<String, String> {
    let file = std::fs::File::open("src/mappings.yaml").unwrap();
    let mapping: HashMap<String, String> = serde_yaml::from_reader(file).unwrap();
    println!("Read YAML mapping: {:?}", mapping);
    mapping
}
