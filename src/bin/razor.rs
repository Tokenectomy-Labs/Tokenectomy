#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokenectomy::run_cli().await
}
