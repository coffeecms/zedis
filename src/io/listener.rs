use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;

#[allow(dead_code)]
pub struct ZedisListener {
    listener: TcpListener,
}

#[allow(dead_code)]
impl ZedisListener {
    pub async fn bind(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { listener })
    }

    pub async fn accept(&self) -> Result<(TcpStream, std::net::SocketAddr)> {
        let (socket, addr) = self.listener.accept().await?;
        // Set TCP_NODELAY for lower latency
        socket.set_nodelay(true)?;
        Ok((socket, addr))
    }
}
