#![allow(dead_code)]
#![allow(non_snake_case)]

#[cfg(test)]
mod test {
    // On global type
    #[register_events]
    pub enum Events {
        #[host_to_guest]
        Html,
    }

    // On host

    receive!(|msg| match msg {
        Events::Html(html) => {
            println!("HTML: {}", html);
        }
    });
    send!(Events::Html, "/u/123123");

    static IMAGE: Image = create_image!(Bytes::new(), Events);

    // On guest

    fn main() {
        loop {
            receive!(|msg| match msg {
                Events::Html(url) => {
                    println!("URL: {}", url);
                    "<h1>Hello</h1>".to_string()
                }
            });
        }
    }

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {}
}
