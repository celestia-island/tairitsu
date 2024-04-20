mod html;
mod model;

use anyhow::Result;
use wasi::sockets::{
    instance_network::instance_network,
    ip_name_lookup::{resolve_addresses, IpAddress},
    network::{IpAddressFamily, Ipv4SocketAddress},
    tcp::IpSocketAddress,
    tcp_create_socket::create_tcp_socket,
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
    let request = resolve_addresses(&instance_network(), "localhost")?;
    let ip = loop {
        if let Ok(ret) = request.resolve_next_address() {
            println!(
                "{}",
                serde_json::to_string(&Msg::new("debug", format!("IP: {:?}", ret))).unwrap()
            );

            match ret {
                Some(IpAddress::Ipv4(ip)) => break ip,
                _ => unimplemented!("Only IPv4 is supported"),
            }
        }
    };
    let request = create_tcp_socket(IpAddressFamily::Ipv4)?;
    loop {
        if request
            .start_connect(
                &instance_network(),
                IpSocketAddress::Ipv4(Ipv4SocketAddress {
                    address: ip,
                    port: 8080,
                }),
            )
            .is_ok()
        {
            break;
        }
    }
    let (input, output) = loop {
        if let Ok(ret) = request.finish_connect() {
            break ret;
        }
    };
    output.write(b"GET / HTTP/1.1\r\n")?;
    output.write(b"Host: localhost\r\n")?;
    output.write(b"\r\n")?;
    output.flush()?;
    let response = loop {
        // FIXME - Only read 100000 bytes at first
        let response = input.read(100000)?;
        if response.len() > 0 && response[0] != 0 {
            break response;
        }
    };
    let response = String::from_utf8_lossy(&response).to_string();
    println!(
        "{}",
        serde_json::to_string(&Msg::new("debug", format!("Response: {:?}", response))).unwrap()
    );

    Ok(())
}
