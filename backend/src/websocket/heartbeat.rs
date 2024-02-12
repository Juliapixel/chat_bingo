use std::{task::Poll, time::{Duration, Instant}};

use actix::{Actor, ActorContext, ActorFuture};
use actix_web_actors::ws::{CloseCode, CloseReason};

use super::BingoWs;

pub struct Heartbeat {
    interval: tokio::time::Interval,
    leniency: Duration,
}

impl Heartbeat {
    pub fn new(interval: Duration, leniency_time: Duration) -> Self {
        Self {
            interval: tokio::time::interval(interval),
            leniency: leniency_time
        }
    }
}

impl ActorFuture<BingoWs> for Heartbeat {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        srv: &mut BingoWs,
        ctx: &mut <BingoWs as Actor>::Context,
        task: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.interval.poll_tick(task) {
            std::task::Poll::Ready(_) => {
                if Instant::now() - srv.last_message > self.interval.period() + self.leniency {
                    ctx.close(Some(CloseReason{ code: CloseCode::Abnormal, description: Some("timed out".into()) }));
                    ctx.stop();
                    Poll::Ready(())
                } else {
                    ctx.ping(b"blorginton");
                    Poll::Pending
                }
            },
            Poll::Pending => return Poll::Pending,
        }
    }
}
