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

    const TIMEOUT_DURATION: u64 = 5;


    #[tokio::test]
    async fn case_01_connect() -> anyhow::Result<()> {

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        tokio::spawn(async move { space_race::run_with_listener(listener).await });

        test!(tokio::net::TcpStream::connect(std::net::SocketAddr::V4(
            std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, port),
        )))?;

        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    #[tokio::test]
    async fn case_02_websocket_handshake() -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        tokio::spawn(async move { space_race::run_with_listener(listener).await });

        let stream = tokio::net::TcpStream::connect(std::net::SocketAddr::V4(
            std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, port))).await?;


        // let request = http::Request::builder()
        //     .method("GET")
        //     .uri(format!("ws://localhost:{}/", port))
        //     .body(())
        //     .unwrap();

        tokio_tungstenite::client_async(format!("ws://127.0.0.1:{}", port), stream)
            .await?;

        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }
}
