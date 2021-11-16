use parking_lot::Mutex;
use pin_project_lite::pin_project;
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pin_project! {
    // This type can be wrapped around any IO stream to record its read buffer.
    pub(crate) struct RecordStream<I> {
        #[pin]
        inner: I,
        buf: Arc<Mutex<Vec<u8>>>,
    }
}

impl<I> RecordStream<I> {
    pub(crate) fn new(inner: I, buf: Arc<Mutex<Vec<u8>>>) -> Self {
        Self { inner, buf }
    }
}

impl<I: AsyncRead> AsyncRead for RecordStream<I> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        let poll_result = this.inner.poll_read(cx, buf);

        // Record request frame in designated buffer.
        let mut b = this.buf.lock();
        b.extend(buf.filled().iter());

        poll_result
    }
}

impl<I: AsyncWrite> AsyncWrite for RecordStream<I> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_shutdown(cx)
    }
}
