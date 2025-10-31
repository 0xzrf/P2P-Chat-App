use chat_app::Client;

#[tokio::main]
async fn main() {
    Client::build_and_run().await.unwrap();
}
