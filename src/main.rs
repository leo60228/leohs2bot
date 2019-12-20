use leohs2bot::*;
use surf::Client;

#[async_std::main]
async fn main() {
    let client = Client::new();
    println!("{:?}", token(&client).await);
}
