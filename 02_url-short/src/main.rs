use std::collections::HashMap;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, header, StatusCode};
use hyper::body::Bytes;
use tokio::net::TcpListener;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(redirect))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn redirect(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let path = req.uri().path();
    println!("incoming requests for path {}", path);
    let target_url = get_target_url(path).await?;
    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, target_url)
        .body(Full::new(Bytes::default()))
        .unwrap())
}

async fn get_target_url(source_path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let contents = tokio::fs::read("src/mappings.yaml").await?;
    let mapping: HashMap<String, String> = serde_yaml::from_slice(&contents)?;
    println!("Read YAML mapping: {:?}", mapping);
    let default = String::from("https://beneck.de");
    let result = mapping.get(source_path).unwrap_or(&default);
    Ok(result.clone())
}
