mod html;
mod model;

use anyhow::Result;

use tairitsu_utils::types::proto::backend::Msg;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let html_raw = html::render::render().await?;
    println!(
        "{}",
        serde_json::to_string(&Msg::new("debug", html_raw)).unwrap()
    );
    model::init().await;

    Ok(())
}
