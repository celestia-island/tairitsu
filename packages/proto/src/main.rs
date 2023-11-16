mod html;
mod model;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    model::init().await;
}
