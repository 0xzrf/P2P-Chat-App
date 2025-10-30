use client_display::Client;

#[tokio::main]
async fn main() {
    Client::build().await.unwrap();
}
