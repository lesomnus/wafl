use super::state::{Key, State};
use async_trait::async_trait;
use pingora::{apps::ServerApp, protocols::Stream, server::ShutdownWatch};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub(super) struct Tcp {
    pub state: Arc<State>,
    pub grace: u64,
}
#[async_trait]
impl ServerApp for Tcp {
    async fn process_new(
        self: &Arc<Self>,
        mut stream: Stream,
        shutdown: &ShutdownWatch,
    ) -> Option<Stream> {
        let port = stream.get_socket_digest()?.local_addr()?.as_inet()?.port();
        let mut shutdown = shutdown.clone();
        if *shutdown.borrow() {
            return None;
        }
        let work = async {
            let target = self.state.target(Key::Port(u32::from(port)), None);
            tokio::pin!(target);
            // Detect an early client FIN/reset without waiting for client bytes:
            // server-first protocols still proceed when the target becomes ready.
            let mut prefix = [0u8; 1];
            let (target, prefix) = tokio::select! {
                target=&mut target=>(target.ok()?,None),
                read=stream.read(&mut prefix)=>{
                    let size=read.ok()?;
                    (target.await.ok()?,if size==0{None}else{Some(prefix)})
                }
            };
            let mut backend = tokio::time::timeout(
                Duration::from_secs(5),
                TcpStream::connect(&target.endpoint.address),
            )
            .await
            .ok()?
            .ok()?;
            backend.set_nodelay(true).ok()?;
            if let Some(prefix) = prefix {
                backend.write_all(&prefix).await.ok()?;
            }
            // Tokio propagates EOF with write-half shutdown while the opposite
            // direction keeps running. A reset/error drops both streams.
            let _ = tokio::io::copy_bidirectional(&mut stream, &mut backend).await;
            drop(backend);
            drop(stream);
            drop(target);
            Some(())
        };
        tokio::pin!(work);
        tokio::select! {
            _=&mut work=>{},
            _=shutdown.changed()=>{
                let _=tokio::time::timeout(Duration::from_secs(self.grace),&mut work).await;
            }
        }
        None
    }
}
