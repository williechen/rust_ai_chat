use any_spawner::Executor;

#[tokio::main]
async fn main() {
    let _ = Executor::init_tokio();

    if let Err(err) = server::application().await {
        eprintln!("Server failed to start: {}", err);
    }
}
