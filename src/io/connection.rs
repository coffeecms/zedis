use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use anyhow::Result;
use crate::core::protocol::RespFrame;


pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    /// Reads a frame from the connection.
    pub async fn read_frame(&mut self) -> Result<Option<RespFrame>> {
        loop {
            // Attempt to parse a frame from the buffered data
            if let Ok((remaining, frame)) = crate::core::protocol::parse_frame(&self.buffer) {
                // Calculate how many bytes were consumed
                let consumed = self.buffer.len() - remaining.len();
                // Advance the buffer
                let _ = self.buffer.split_to(consumed);
                return Ok(Some(frame));
            }

            // If incomplete, read more data from the socket
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this, it means no more data.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                     return Err(anyhow::anyhow!("connection reset by peer"));
                }
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &RespFrame) -> Result<()> {
        let mut buf = Vec::new();
        frame.encode(&mut buf);
        self.stream.write_all(&buf).await?;
        self.stream.flush().await?;
        Ok(())
    }
}

