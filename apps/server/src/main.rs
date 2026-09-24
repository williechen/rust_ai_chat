#[tokio::main]
async fn main() {
    if let Err(err) = server::application().await {
        eprintln!("Server failed to start: {}", err);
    }
}
