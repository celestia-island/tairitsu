use bytes::Bytes;
use flume::Sender;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use wasmtime_wasi::preview2::{
    HostInputStream, HostOutputStream, StdinStream, StdoutStream, StreamResult, Subscribe,
};

pub struct InputStream<Res>
where
    Res: Clone + Serialize + Send + Sync,
{
    pub tasks: Arc<Mutex<Vec<Res>>>,
}

#[async_trait::async_trait]
impl<Res> Subscribe for InputStream<Res>
where
    Res: Clone + Serialize + Send + Sync,
{
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl<Res> HostInputStream for InputStream<Res>
where
    Res: Clone + Serialize + Send + Sync,
{
    fn read(&mut self, _size: usize) -> StreamResult<Bytes> {
        loop {
            {
                let mut tasks = self.tasks.lock().unwrap();
                if tasks.len() > 0 {
                    let ret = tasks.remove(0);
                    let ret = serde_json::to_string(&ret).unwrap() + "\n";
                    let ret = Bytes::from(ret);

                    return Ok(ret);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

pub struct OutputStream<'de, Req>
where
    Req: Clone + Deserialize<'de> + Send + Sync,
{
    pub tx: Sender<Req>,
    phantom: std::marker::PhantomData<&'de ()>,
}

#[async_trait::async_trait]
impl<'de, Req> Subscribe for OutputStream<'de, Req>
where
    Req: Clone + Deserialize<'de> + Send + Sync,
{
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl<'de, Req> HostOutputStream for OutputStream<'de, Req>
where
    Req: Clone + Clone + Deserialize<'de> + Send + Sync,
{
    fn write<'a>(&mut self, bytes: Bytes) -> StreamResult<()> {
        let bytes = bytes.clone();
        let msg = serde_json::from_slice::<Req>(&bytes.as_ref()).expect("Failed to parse message");

        self.tx.send(msg).expect("Failed to send message");
        Ok(())
    }

    fn flush(&mut self) -> StreamResult<()> {
        Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        Ok(8192)
    }
}

pub struct HostInputStreamBox<Res>
where
    Res: Clone + Serialize + Send + Sync,
{
    pub tasks: Arc<Mutex<Vec<Res>>>,
}

impl<Res> StdinStream for HostInputStreamBox<Res>
where
    Res: Clone + Serialize + Send + Sync,
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

pub struct HostOutputStreamBox<'de, Req>
where
    Req: Clone + Deserialize<'de> + Send + Sync,
{
    pub tx: Sender<Req>,
    phantom: std::marker::PhantomData<&'de ()>,
}

impl<'de, Req> HostOutputStreamBox<'de, Req>
where
    Req: Clone + Deserialize<'de> + Send + Sync,
{
    pub fn new(tx: Sender<Req>) -> Self {
        Self {
            tx,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'de, Req> StdoutStream for HostOutputStreamBox<'de, Req>
where
    Req: Clone + Deserialize<'de> + Send + Sync,
{
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(OutputStream {
            tx: self.tx.clone(),
            phantom: std::marker::PhantomData,
        })
    }

    fn isatty(&self) -> bool {
        false
    }
}
