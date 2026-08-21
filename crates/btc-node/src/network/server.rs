use std::{io, sync::Arc};

use tokio::{net::TcpListener, sync::RwLock};

use crate::network::PeerManager;

pub struct NetworkServer {
    listener: TcpListener,
}

impl NetworkServer {
    pub async fn bind(address: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(address).await?;

        Ok(Self { listener })
    }

    pub async fn accept(&self) -> io::Result<(tokio::net::TcpStream, std::net::SocketAddr)> {
        self.listener.accept().await
    }

    pub async fn run(&self, peer_manager: Arc<RwLock<PeerManager>>) -> io::Result<()> {
        loop {
            let (stream, address) = self.listener.accept().await?;

            let peer_manager = Arc::clone(&peer_manager);

            tokio::spawn(async move {
                PeerManager::process_connection(peer_manager, stream, address).await;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    #[tokio::test]
    async fn server_can_read_and_write() {
        let server = NetworkServer::bind("127.0.0.1:18444").await.unwrap();

        let mut client = TcpStream::connect("127.0.0.1:18444").await.unwrap();

        let (mut server_stream, _) = server.accept().await.unwrap();

        // client -> server
        client.write_all(b"hello").await.unwrap();

        let mut buffer = [0u8; 5];

        server_stream.read_exact(&mut buffer).await.unwrap();

        assert_eq!(&buffer, b"hello");

        // server -> client
        server_stream.write_all(b"world").await.unwrap();

        let mut buffer = [0u8; 5];

        client.read_exact(&mut buffer).await.unwrap();

        assert_eq!(&buffer, b"world");
    }
}
