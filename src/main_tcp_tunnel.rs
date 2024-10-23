use tokio::io::copy;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};

use futures::FutureExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let listen_addr = "127.0.0.1:443".to_string();
    let server_addr = "www.google.com:443".to_string();

    println!("Listening on: {}", listen_addr);
    println!("Proxying to: {}", server_addr);

    let listener = TcpListener::bind(listen_addr).await?;

    while let Ok((mut inbound, _)) = listener.accept().await {
        let mut outbound = TcpStream::connect(server_addr.clone()).await?;

        // let (mut in_recv, mut in_send) = inbound.split();
        // let (mut out_recv, mut out_send) = outbound.split();

        // let t1 = copy(&mut in_recv, &mut out_send);
        // let t2 = copy(&mut out_recv, &mut in_send);

        // tokio::join!(t1, t2);
        // println!("done");

        tokio::spawn(async move {
            copy_bidirectional(&mut inbound, &mut outbound)
                .map(|r| {
                    if let Err(e) = r {
                        println!("Failed to transfer; error={}", e);
                    }
                })
                .await
        });
    }

    Ok(())
}