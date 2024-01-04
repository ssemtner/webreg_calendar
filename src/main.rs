mod calendar;
mod parse;
mod server;
mod types;

#[tokio::main]
async fn main() {
    server::serve().await;
}
