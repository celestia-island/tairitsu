use bytes::Bytes;
use flume::Sender;
use std::sync::{Arc, Mutex};

use wasmtime_wasi::preview2::{
    HostInputStream, HostOutputStream, StdinStream, StdoutStream, StreamError, StreamResult,
    Subscribe,
};

use tairitsu_utils::types::proto::backend::Msg;

pub struct InputStream {
    pub tasks: Arc<Mutex<Vec<Msg>>>,
}

#[async_trait::async_trait]
impl Subscribe for InputStream {
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl HostInputStream for InputStream {
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

pub struct OutputStream {
    pub tx: Sender<Msg>,
    pub buffer: String,
}

#[async_trait::async_trait]
impl Subscribe for OutputStream {
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl HostOutputStream for OutputStream {
    fn write(&mut self, bytes: Bytes) -> StreamResult<()> {
        let msg = String::from_utf8(bytes.to_vec()).expect("Failed to receive message");
        self.buffer.push_str(&msg);

        // Check if the last character is '\n'
        if !self.buffer.ends_with('\n') {
            return Ok(());
        }

        if let Ok(msg) = serde_json::from_str::<Msg>(&self.buffer) {
            self.tx.send(msg).expect("Failed to send message");
            self.buffer.clear();
            Ok(())
        } else {
            Err(StreamError::LastOperationFailed(anyhow::anyhow!(
                "Failed to parse message"
            )))
        }
    }

    fn flush(&mut self) -> StreamResult<()> {
        Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        // Fixed by wasmtime's wasi implementation.
        Ok(4096)
    }
}

pub struct HostInputStreamBox {
    pub tasks: Arc<Mutex<Vec<Msg>>>,
}

impl StdinStream for HostInputStreamBox {
    fn stream(&self) -> Box<dyn HostInputStream> {
        Box::new(InputStream {
            tasks: self.tasks.clone(),
        })
    }

    fn isatty(&self) -> bool {
        false
    }
}

pub struct HostOutputStreamBox {
    pub tx: Sender<Msg>,
}

impl StdoutStream for HostOutputStreamBox {
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(OutputStream {
            tx: self.tx.clone(),
            buffer: String::new(),
        })
    }

    fn isatty(&self) -> bool {
        false
    }
}
