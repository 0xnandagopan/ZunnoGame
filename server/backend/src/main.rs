#[tokio::main]
async fn main() -> anyhow::Result<()> {
    zunnogame_backend::run().await
}
