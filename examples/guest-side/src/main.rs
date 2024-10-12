mod html;
mod model;

use anyhow::Result;
use std::io::Read;
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
    let request = resolve_addresses(&instance_network(), "httpbin.org")?;
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
                    port: 80,
                }),
            )
            .is_ok()
        {
            break;
        }
    }
    let (mut input, output) = loop {
        if let Ok(ret) = request.finish_connect() {
            break ret;
        }
    };
    output.write(b"GET /get HTTP/1.1\r\n")?;
    output.write(b"Host: httpbin.org\r\n")?;
    output.write(b"\r\n")?;
    output.flush()?;
    println!(
        "{}",
        serde_json::to_string(&Msg::new("debug", "Sent request".to_string())).unwrap()
    );
    let response = {
        let mut ret = "".to_string();
        input.read_to_string(&mut ret)?;
        ret
    };
    println!(
        "{}",
        serde_json::to_string(&Msg::new("debug", format!("Response: {:?}", response))).unwrap()
    );

    Ok(())
}
