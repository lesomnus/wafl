use crate::proto::wafl::{Channel, Log};
use bytes::Bytes;
use connectrpc::ConnectError;
use futures_util::Stream;
use std::{
    collections::VecDeque,
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{io::AsyncWrite, sync::watch};
use wasmtime_wasi::{
    cli::{IsTerminal, StdoutStream},
    p2::{OutputStream, Pollable, StreamError},
};

const BYTES: usize = 1024 * 1024;
const RECORDS: usize = 4096;
const CHUNK: usize = 8192;
pub(crate) type LogStream = Pin<Box<dyn Stream<Item = Result<Log, ConnectError>> + Send>>;

pub(crate) struct Logs {
    state: Mutex<State>,
    changed: watch::Sender<()>,
}
#[derive(Default)]
struct State {
    entries: VecDeque<Log>,
    bytes: usize,
    sequence: u64,
    closed: bool,
}
impl Logs {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State::default()),
            changed: watch::channel(()).0,
        })
    }
    pub fn sink(self: &Arc<Self>, generation: u64, channel: Channel) -> Sink {
        Sink {
            logs: self.clone(),
            generation,
            channel,
        }
    }
    fn push(&self, generation: u64, channel: Channel, bytes: &[u8]) {
        let mut state = self.state.lock().unwrap();
        if state.closed {
            return;
        }
        let date_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        for chunk in bytes.chunks(CHUNK) {
            state.sequence += 1;
            let entry = Log {
                sequence: state.sequence,
                generation,
                channel: channel.into(),
                data: chunk.to_vec(),
                date_ms,
                ..Default::default()
            };
            state.bytes += chunk.len();
            state.entries.push_back(entry);
            while state.bytes > BYTES || state.entries.len() > RECORDS {
                state.bytes -= state.entries.pop_front().unwrap().data.len();
            }
        }
        drop(state);
        self.changed.send_replace(());
    }
    pub fn close(&self) {
        self.state.lock().unwrap().closed = true;
        self.changed.send_replace(());
    }
    pub fn stderr(&self, generation: u64) -> String {
        let state = self.state.lock().unwrap();
        let mut bytes = Vec::new();
        for entry in state
            .entries
            .iter()
            .rev()
            .filter(|e| e.generation == generation && e.channel == Channel::Stderr)
        {
            for byte in entry.data.iter().rev().take(16_384 - bytes.len()) {
                bytes.push(*byte);
            }
            if bytes.len() == 16_384 {
                break;
            }
        }
        bytes.reverse();
        String::from_utf8_lossy(&bytes).into_owned()
    }
    fn read(&self, cursor: u64, history: bool) -> Result<(Vec<Log>, bool, u64), ConnectError> {
        let state = self.state.lock().unwrap();
        if cursor > state.sequence {
            return Err(ConnectError::invalid_argument(
                "log cursor is beyond the latest sequence",
            ));
        }
        if (!history || cursor != 0)
            && state
                .entries
                .front()
                .is_some_and(|entry| cursor + 1 < entry.sequence)
        {
            return Err(ConnectError::out_of_range(
                "log cursor expired; read retained history with cursor zero",
            ));
        }
        Ok((
            state
                .entries
                .iter()
                .filter(|e| e.sequence > cursor)
                .cloned()
                .collect(),
            state.closed,
            state.sequence,
        ))
    }
    pub fn tail(
        self: &Arc<Self>,
        cursor: u64,
        limit: u32,
        follow: bool,
    ) -> Result<LogStream, ConnectError> {
        if limit as usize > RECORDS {
            return Err(ConnectError::invalid_argument(
                "log history limit is at most 4096 records",
            ));
        }
        // Subscribe before capturing history so writes cannot fall into a gap.
        let mut changed = self.changed.subscribe();
        let (mut history, closed, mut cursor) = self.read(cursor, true)?;
        if limit != 0 && history.len() > limit as usize {
            history.drain(..history.len() - limit as usize);
        }
        let logs = self.clone();
        Ok(Box::pin(async_stream::try_stream! {
            for entry in history { yield entry; }
            if follow && !closed {
                loop {
                    let (batch, closed, sequence) = logs.read(cursor, false)?;
                    cursor = sequence;
                    for entry in batch { yield entry; }
                    if closed { break; }
                    if changed.changed().await.is_err() { break; }
                }
            }
        }))
    }
}

#[derive(Clone)]
pub(crate) struct Sink {
    logs: Arc<Logs>,
    generation: u64,
    channel: Channel,
}
impl IsTerminal for Sink {
    fn is_terminal(&self) -> bool {
        false
    }
}
impl StdoutStream for Sink {
    fn async_stream(&self) -> Box<dyn AsyncWrite + Send + Sync> {
        Box::new(self.clone())
    }
    fn p2_stream(&self) -> Box<dyn OutputStream> {
        Box::new(self.clone())
    }
}
#[async_trait::async_trait]
impl Pollable for Sink {
    async fn ready(&mut self) {}
}
#[async_trait::async_trait]
impl OutputStream for Sink {
    fn write(&mut self, bytes: Bytes) -> Result<(), StreamError> {
        self.logs.push(self.generation, self.channel, &bytes);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), StreamError> {
        Ok(())
    }
    fn check_write(&mut self) -> Result<usize, StreamError> {
        Ok(64 * 1024)
    }
}
impl AsyncWrite for Sink {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.logs.push(self.generation, self.channel, bytes);
        Poll::Ready(Ok(bytes.len()))
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    #[tokio::test]
    async fn history_follow_resume_retention_and_close() {
        let logs = Logs::new();
        logs.push(1, Channel::Stdout, b"first\n");
        let mut follower = logs.tail(0, 0, true).unwrap();
        assert_eq!(follower.next().await.unwrap().unwrap().sequence, 1);
        logs.push(2, Channel::Stderr, b"second\n");
        assert_eq!(follower.next().await.unwrap().unwrap().generation, 2);
        let history: Vec<_> = logs.tail(1, 0, false).unwrap().collect().await;
        assert_eq!(history.len(), 1);
        logs.push(2, Channel::Stdout, &vec![42; BYTES * 2]);
        assert_eq!(
            follower.next().await.unwrap().unwrap_err().code,
            connectrpc::ErrorCode::OutOfRange
        );
        assert!(logs.state.lock().unwrap().bytes <= BYTES);
        let mut follower = logs.tail(0, 1, true).unwrap();
        assert!(follower.next().await.unwrap().is_ok());
        logs.close();
        assert!(follower.next().await.is_none());
    }
}
