#[cfg(test)]
use test_helpers_async::*;

#[before_all]
#[cfg(test)]
mod all {

    use futures_timeout::TimeoutExt;
    use tokio::time::Duration;

    #[macro_export]
    macro_rules! test {
        ( $x:expr ) => {{
            $x.timeout(Duration::from_secs(TIMEOUT_DURATION)).await?
        }};
    }

    pub fn before_all() {
    }

    const TIMEOUT_DURATION: u64 = 20;


    #[tokio::test]
    async fn case_01_connect() -> anyhow::Result<()> {

        let socket = tokio::net::TcpSocket::new_v4()?;

        tokio::spawn(async {
            space_race::run("127.0.0.1:8080").await;
        });

        test!(socket.connect(std::net::SocketAddr::V4(
            std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 8080),
        )))?;
        Ok(())
    }

    #[tokio::test]
    async fn case_02_websocket_handshake() -> anyhow::Result<()> {
        let socket = tokio::net::TcpSocket::new_v4()?;

        tokio::spawn(async {
            space_race::run("127.0.0.1:8080").await;
        });

        socket.connect(std::net::SocketAddr::V4(
            std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 8080),
        ))
        .await?;

        let (ws_stream, _) = tokio_tungstenite::accept_async(socket).await?;
        Ok(())
    }
}
