use bytes::Bytes;
use flume::Sender;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use wasmtime_wasi::preview2::{
    HostInputStream, HostOutputStream, StdinStream, StdoutStream, StreamResult, Subscribe,
};

pub struct InputStream<'a, Res>
where
    Res: Serialize + Deserialize<'static> + Send + Sync,
{
    pub tasks: Arc<Mutex<Vec<&'a Res>>>,
}

#[async_trait::async_trait]
impl<'a, Res> Subscribe for InputStream<'static, Res>
where
    Res: Serialize + Deserialize<'static> + Send + Sync,
{
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl<'a, Res> HostInputStream for InputStream<'static, Res>
where
    Res: Serialize + Deserialize<'static> + Send + Sync,
{
    fn read(&mut self, _size: usize) -> StreamResult<Bytes> {
        loop {
            {
                let mut tasks = self.tasks.lock().unwrap();
                if tasks.len() > 0 {
                    let ret = tasks.remove(0);
                    let ret = serde_json::to_string(ret).unwrap() + "\n";
                    let ret = Bytes::from(ret);

                    return Ok(ret);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

pub struct OutputStream<'a, Req>
where
    Req: Serialize + Deserialize<'static> + Send + Sync,
{
    pub tx: Sender<&'a Req>,
}

#[async_trait::async_trait]
impl<'a, Req> Subscribe for OutputStream<'static, Req>
where
    Req: Serialize + Deserialize<'static> + Send + Sync,
{
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl<'a, Req> HostOutputStream for OutputStream<'static, Req>
where
    Req: Serialize + Deserialize<'static> + Send + Sync,
{
    fn write(&mut self, bytes: Bytes) -> StreamResult<()> {
        let msg = String::from_utf8(bytes.to_vec()).expect("Failed to parse message");
        let msg = serde_json::from_str::<Req>(&msg).expect("Failed to parse message");

        self.tx.send(&msg).expect("Failed to send message");
        Ok(())
    }

    fn flush(&mut self) -> StreamResult<()> {
        Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        Ok(8192)
    }
}

pub struct HostInputStreamBox<'a, Res>
where
    Res: Serialize + Deserialize<'static> + Send + Sync,
{
    pub tasks: Arc<Mutex<Vec<&'a Res>>>,
}

impl<'a, Res> StdinStream for HostInputStreamBox<'a, Res>
where
    Res: Serialize + Deserialize<'static> + Send + Sync,
{
    fn stream(&self) -> Box<dyn HostInputStream> {
        Box::new(InputStream {
            tasks: self.tasks.clone(),
        })
    }

    fn isatty(&self) -> bool {
        false
    }
}

pub struct HostOutputStreamBox<'a, Req: Serialize> {
    pub tx: Sender<&'a Req>,
}

impl<'a, Req> StdoutStream for HostOutputStreamBox<'a, Req>
where
    Req: Serialize + Deserialize<'static> + Send + Sync,
{
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(OutputStream {
            tx: self.tx.clone(),
        })
    }

    fn isatty(&self) -> bool {
        false
    }
}
