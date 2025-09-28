use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;
    client.set("hello", "value".into()).await?;
    let res = client.get("hello").await?;
    println!("got value from the server: result={:?}", res);
    Ok(())
}
