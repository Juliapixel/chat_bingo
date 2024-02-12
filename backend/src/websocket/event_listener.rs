use std::{future::Future, pin::Pin, task::Poll};

use actix::{Actor, ActorFuture};
use log::error;
use tokio::sync::broadcast::Receiver;

use crate::event::ServerEvent;

use super::BingoWs;

pub struct EventListener {
    rx: Receiver<ServerEvent>
}

impl EventListener {
    pub fn new(rx: Receiver<ServerEvent>) -> Self {
        Self {
            rx
        }
    }
}

impl ActorFuture<BingoWs> for EventListener {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _srv: &mut BingoWs,
        ctx: &mut <BingoWs as Actor>::Context,
        task: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut recv_fut = Box::pin(async { self.rx.recv().await });
        let recv_fut = Pin::new(&mut recv_fut);

        match Future::poll(recv_fut, task) {
            Poll::Ready(Ok(e)) => {
                ctx.text(serde_json::to_string(&e).unwrap());
                Poll::Pending
            },
            Poll::Ready(Err(err)) => {
                error!("{err}");
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
