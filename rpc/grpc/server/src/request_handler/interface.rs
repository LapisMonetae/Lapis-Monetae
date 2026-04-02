use super::method::{DropFn, Method, MethodTrait, RoutingPolicy};
use crate::{
    connection::Connection,
    connection_handler::ServerContext,
    error::{GrpcServerError, GrpcServerResult},
};
use lmt_grpc_core::{
    ops::LmtdPayloadOps,
    protowire::{LmtdRequest, LmtdResponse},
};
use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

pub type LmtdMethod = Method<ServerContext, Connection, LmtdRequest, LmtdResponse>;
pub type DynLmtdMethod = Arc<dyn MethodTrait<ServerContext, Connection, LmtdRequest, LmtdResponse>>;
pub type LmtdDropFn = DropFn<LmtdRequest, LmtdResponse>;
pub type LmtdRoutingPolicy = RoutingPolicy<LmtdRequest, LmtdResponse>;

/// An interface providing methods implementations and a fallback "not implemented" method
/// actually returning a message with a "not implemented" error.
///
/// The interface can provide a method clone for every [`LmtdPayloadOps`] variant for later
/// processing of related requests.
///
/// It is also possible to directly let the interface itself process a request by invoking
/// the `call()` method.
pub struct Interface {
    server_ctx: ServerContext,
    methods: HashMap<LmtdPayloadOps, DynLmtdMethod>,
    method_not_implemented: DynLmtdMethod,
}

impl Interface {
    pub fn new(server_ctx: ServerContext) -> Self {
        let method_not_implemented = Arc::new(Method::new(|_, _, lmtd_request: LmtdRequest| {
            Box::pin(async move {
                match lmtd_request.payload {
                    Some(ref request) => Ok(LmtdResponse {
                        id: lmtd_request.id,
                        payload: Some(LmtdPayloadOps::from(request).to_error_response(GrpcServerError::MethodNotImplemented.into())),
                    }),
                    None => Err(GrpcServerError::InvalidRequestPayload),
                }
            })
        }));
        Self { server_ctx, methods: Default::default(), method_not_implemented }
    }

    pub fn method(&mut self, op: LmtdPayloadOps, method: LmtdMethod) {
        let method: DynLmtdMethod = Arc::new(method);
        if self.methods.insert(op, method).is_some() {
            panic!("RPC method {op:?} is declared multiple times")
        }
    }

    pub fn replace_method(&mut self, op: LmtdPayloadOps, method: LmtdMethod) {
        let method: DynLmtdMethod = Arc::new(method);
        let _ = self.methods.insert(op, method);
    }

    pub fn set_method_properties(&mut self, op: LmtdPayloadOps, tasks: usize, queue_size: usize, routing_policy: LmtdRoutingPolicy) {
        self.methods.entry(op).and_modify(|x| {
            let method: Method<ServerContext, Connection, LmtdRequest, LmtdResponse> =
                Method::with_properties(x.method_fn(), tasks, queue_size, routing_policy);
            let method: Arc<dyn MethodTrait<ServerContext, Connection, LmtdRequest, LmtdResponse>> = Arc::new(method);
            *x = method;
        });
    }

    pub async fn call(&self, op: &LmtdPayloadOps, connection: Connection, request: LmtdRequest) -> GrpcServerResult<LmtdResponse> {
        self.methods.get(op).unwrap_or(&self.method_not_implemented).call(self.server_ctx.clone(), connection, request).await
    }

    pub fn get_method(&self, op: &LmtdPayloadOps) -> DynLmtdMethod {
        self.methods.get(op).unwrap_or(&self.method_not_implemented).clone()
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interface").finish()
    }
}
