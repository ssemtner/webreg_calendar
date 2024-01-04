mod calendar;
mod parse;
mod server;
mod types;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    server::create()
}
