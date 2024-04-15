use bytes::Bytes;
use flume::Sender;
use std::sync::{Arc, Mutex};

use wasmtime_wasi::{
    HostInputStream, HostOutputStream, StdinStream, StdoutStream, StreamResult, Subscribe,
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
    pub buffer: Vec<Bytes>,
}

#[async_trait::async_trait]
impl Subscribe for OutputStream {
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl HostOutputStream for OutputStream {
    fn write(&mut self, bytes: Bytes) -> StreamResult<()> {
        self.buffer.push(bytes);

        if let Some(last) = self.buffer.last() {
            // Check if the last character is '\n'
            if last.last() == Some(&b'\n') {
                let bytes_combined = Bytes::from(self.buffer.concat());
                let buffer = String::from_utf8(bytes_combined.to_vec()).unwrap();

                self.buffer.clear();
                if let Ok(msg) = serde_json::from_str::<Msg>(&buffer) {
                    self.tx.send(msg).expect("Failed to send message");
                }
            }
        }

        Ok(())
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
            buffer: vec![],
        })
    }

    fn isatty(&self) -> bool {
        false
    }
}
