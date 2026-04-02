use super::error::Result;
use core::fmt::Debug;
use lmt_grpc_core::{
    ops::LmtdPayloadOps,
    protowire::{LmtdRequest, LmtdResponse},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(crate) mod id;
pub(crate) mod matcher;
pub(crate) mod queue;

pub(crate) trait Resolver: Send + Sync + Debug {
    fn register_request(&self, op: LmtdPayloadOps, request: &LmtdRequest) -> LmtdResponseReceiver;
    fn handle_response(&self, response: LmtdResponse);
    fn remove_expired_requests(&self, timeout: Duration);
}

pub(crate) type DynResolver = Arc<dyn Resolver>;

pub(crate) type LmtdResponseSender = oneshot::Sender<Result<LmtdResponse>>;
pub(crate) type LmtdResponseReceiver = oneshot::Receiver<Result<LmtdResponse>>;
