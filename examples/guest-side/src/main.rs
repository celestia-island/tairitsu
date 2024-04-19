mod html;
mod model;

use anyhow::Result;
use wasi::sockets::{
    instance_network::instance_network,
    ip_name_lookup::{resolve_addresses, IpAddress},
};

use tairitsu_utils::types::proto::backend::Msg;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let html_raw = html::render::render().await?;
    println!(
        "{}",
        serde_json::to_string(&Msg::new("debug", html_raw)).unwrap()
    );
    model::init().await;

    // TODO - This is a temporary solution to make the example work

    println!(
        "{}",
        serde_json::to_string(&Msg::new("debug", "Try to write file")).unwrap()
    );
    std::fs::write("/tmp/test.txt", "Hello, world!")?;

    println!(
        "{}",
        serde_json::to_string(&Msg::new("debug", "Try to send ip lookup")).unwrap()
    );
    let request = resolve_addresses(&instance_network(), "cor-games.com")?;
    loop {
        if let Ok(ret) = request.resolve_next_address() {
            let ret = ret
                .map(|addr| match addr {
                    IpAddress::Ipv4((a, b, c, d)) => format!("{}.{}.{}.{}", a, b, c, d),
                    IpAddress::Ipv6((a, b, c, d, e, f, g, h)) => format!(
                        "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                        a, b, c, d, e, f, g, h
                    ),
                })
                .unwrap_or("Unknown address".to_string());
            println!(
                "{}",
                serde_json::to_string(&Msg::new("debug", ret)).unwrap()
            );
            break;
        }
    }

    Ok(())
}
