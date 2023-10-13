use anyhow::Result;
use bytes::Bytes;

use wasmtime_wasi::preview2::{HostInputStream, HostOutputStream, OutputStreamError, StreamState};

use tairitsu_utils::types::proto::backend::Msg;

pub struct InputStream {}

impl InputStream {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl HostInputStream for InputStream {
    fn read(&mut self, size: usize) -> Result<(Bytes, StreamState)> {
        println!("read {} bytes", size);

        let ret = Msg {
            id: 233,
            data: "hello".to_string(),
        };
        let ret = ron::to_string(&ret).unwrap() + "\n";
        let ret = Bytes::from(ret);

        Ok((ret, StreamState::Open))
    }

    async fn ready(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct OutputStream {}

impl OutputStream {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl HostOutputStream for OutputStream {
    fn write(&mut self, bytes: Bytes) -> Result<(), OutputStreamError> {
        let msg =
            String::from_utf8(bytes.to_vec()).map_err(|e| OutputStreamError::Trap(e.into()))?;
        let msg = ron::from_str::<Msg>(&msg).map_err(|e| OutputStreamError::Trap(e.into()))?;

        println!("{:?}", msg);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), OutputStreamError> {
        Ok(())
    }

    async fn write_ready(&mut self) -> Result<usize, OutputStreamError> {
        Ok(256 * 256)
    }
}
