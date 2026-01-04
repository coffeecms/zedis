use async_trait::async_trait;
use std::io;

#[async_trait]
#[allow(dead_code)]
pub trait AsyncStream: Send + Sync {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
}
