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
        env_logger::init();
    }

    const TIMEOUT_DURATION: u64 = 20;


    #[tokio::test]
    async fn case_01_connect() -> anyhow::Result<()> {

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let hdl = space_race::run_with_listener(listener);

        test!(tokio::net::TcpStream::connect(std::net::SocketAddr::V4(
            std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, port),
        )))?;

        hdl.await;
        Ok(())
    }

    #[tokio::test]
    async fn case_02_websocket_handshake() -> anyhow::Result<()> {
        tokio::spawn(async {
            space_race::run("127.0.0.1", 0).await;
        });

        let stream = tokio::net::TcpStream::connect(std::net::SocketAddr::V4(
            std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 8080),
        ))
        .await?;

        test!(tokio_tungstenite::accept_async(stream))?;

        Ok(())
    }
}
