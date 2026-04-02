use crate::protowire::{lmtd_request, LmtdRequest, LmtdResponse};

impl From<lmtd_request::Payload> for LmtdRequest {
    fn from(item: lmtd_request::Payload) -> Self {
        LmtdRequest { id: 0, payload: Some(item) }
    }
}

impl AsRef<LmtdRequest> for LmtdRequest {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<LmtdResponse> for LmtdResponse {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub mod lmtd_request_convert {
    use crate::protowire::*;
    use lmt_rpc_core::{RpcError, RpcResult};

    impl_into_lmtd_request!(Shutdown);
    impl_into_lmtd_request!(SubmitBlock);
    impl_into_lmtd_request!(GetBlockTemplate);
    impl_into_lmtd_request!(GetBlock);
    impl_into_lmtd_request!(GetInfo);

    impl_into_lmtd_request!(GetCurrentNetwork);
    impl_into_lmtd_request!(GetPeerAddresses);
    impl_into_lmtd_request!(GetSink);
    impl_into_lmtd_request!(GetMempoolEntry);
    impl_into_lmtd_request!(GetMempoolEntries);
    impl_into_lmtd_request!(GetConnectedPeerInfo);
    impl_into_lmtd_request!(AddPeer);
    impl_into_lmtd_request!(SubmitTransaction);
    impl_into_lmtd_request!(SubmitTransactionReplacement);
    impl_into_lmtd_request!(GetSubnetwork);
    impl_into_lmtd_request!(GetVirtualChainFromBlock);
    impl_into_lmtd_request!(GetBlocks);
    impl_into_lmtd_request!(GetBlockCount);
    impl_into_lmtd_request!(GetBlockDagInfo);
    impl_into_lmtd_request!(ResolveFinalityConflict);
    impl_into_lmtd_request!(GetHeaders);
    impl_into_lmtd_request!(GetUtxosByAddresses);
    impl_into_lmtd_request!(GetBalanceByAddress);
    impl_into_lmtd_request!(GetBalancesByAddresses);
    impl_into_lmtd_request!(GetSinkBlueScore);
    impl_into_lmtd_request!(Ban);
    impl_into_lmtd_request!(Unban);
    impl_into_lmtd_request!(EstimateNetworkHashesPerSecond);
    impl_into_lmtd_request!(GetMempoolEntriesByAddresses);
    impl_into_lmtd_request!(GetCoinSupply);
    impl_into_lmtd_request!(Ping);
    impl_into_lmtd_request!(GetMetrics);
    impl_into_lmtd_request!(GetConnections);
    impl_into_lmtd_request!(GetSystemInfo);
    impl_into_lmtd_request!(GetServerInfo);
    impl_into_lmtd_request!(GetSyncStatus);
    impl_into_lmtd_request!(GetDaaScoreTimestampEstimate);
    impl_into_lmtd_request!(GetFeeEstimate);
    impl_into_lmtd_request!(GetFeeEstimateExperimental);
    impl_into_lmtd_request!(GetCurrentBlockColor);
    impl_into_lmtd_request!(GetUtxoReturnAddress);

    impl_into_lmtd_request!(NotifyBlockAdded);
    impl_into_lmtd_request!(NotifyNewBlockTemplate);
    impl_into_lmtd_request!(NotifyUtxosChanged);
    impl_into_lmtd_request!(NotifyPruningPointUtxoSetOverride);
    impl_into_lmtd_request!(NotifyFinalityConflict);
    impl_into_lmtd_request!(NotifyVirtualDaaScoreChanged);
    impl_into_lmtd_request!(NotifyVirtualChainChanged);
    impl_into_lmtd_request!(NotifySinkBlueScoreChanged);

    macro_rules! impl_into_lmtd_request {
        ($name:tt) => {
            paste::paste! {
                impl_into_lmtd_request_ex!(lmt_rpc_core::[<$name Request>],[<$name RequestMessage>],[<$name Request>]);
            }
        };
    }

    use impl_into_lmtd_request;

    macro_rules! impl_into_lmtd_request_ex {
        // ($($core_struct:ident)::+, $($protowire_struct:ident)::+, $($variant:ident)::+) => {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<&$core_struct> for lmtd_request::Payload {
                fn from(item: &$core_struct) -> Self {
                    Self::$variant(item.into())
                }
            }

            impl From<&$core_struct> for LmtdRequest {
                fn from(item: &$core_struct) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<$core_struct> for lmtd_request::Payload {
                fn from(item: $core_struct) -> Self {
                    Self::$variant((&item).into())
                }
            }

            impl From<$core_struct> for LmtdRequest {
                fn from(item: $core_struct) -> Self {
                    Self { id: 0, payload: Some((&item).into()) }
                }
            }

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&lmtd_request::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &lmtd_request::Payload) -> RpcResult<Self> {
                    if let lmtd_request::Payload::$variant(request) = item {
                        request.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&LmtdRequest> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &LmtdRequest) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("LmtRequest".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }

            impl From<$protowire_struct> for LmtdRequest {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(lmtd_request::Payload::$variant(item)) }
                }
            }

            impl From<$protowire_struct> for lmtd_request::Payload {
                fn from(item: $protowire_struct) -> Self {
                    lmtd_request::Payload::$variant(item)
                }
            }
        };
    }
    use impl_into_lmtd_request_ex;
}

pub mod lmtd_response_convert {
    use crate::protowire::*;
    use lmt_rpc_core::{RpcError, RpcResult};

    impl_into_lmtd_response!(Shutdown);
    impl_into_lmtd_response!(SubmitBlock);
    impl_into_lmtd_response!(GetBlockTemplate);
    impl_into_lmtd_response!(GetBlock);
    impl_into_lmtd_response!(GetInfo);
    impl_into_lmtd_response!(GetCurrentNetwork);

    impl_into_lmtd_response!(GetPeerAddresses);
    impl_into_lmtd_response!(GetSink);
    impl_into_lmtd_response!(GetMempoolEntry);
    impl_into_lmtd_response!(GetMempoolEntries);
    impl_into_lmtd_response!(GetConnectedPeerInfo);
    impl_into_lmtd_response!(AddPeer);
    impl_into_lmtd_response!(SubmitTransaction);
    impl_into_lmtd_response!(SubmitTransactionReplacement);
    impl_into_lmtd_response!(GetSubnetwork);
    impl_into_lmtd_response!(GetVirtualChainFromBlock);
    impl_into_lmtd_response!(GetBlocks);
    impl_into_lmtd_response!(GetBlockCount);
    impl_into_lmtd_response!(GetBlockDagInfo);
    impl_into_lmtd_response!(ResolveFinalityConflict);
    impl_into_lmtd_response!(GetHeaders);
    impl_into_lmtd_response!(GetUtxosByAddresses);
    impl_into_lmtd_response!(GetBalanceByAddress);
    impl_into_lmtd_response!(GetBalancesByAddresses);
    impl_into_lmtd_response!(GetSinkBlueScore);
    impl_into_lmtd_response!(Ban);
    impl_into_lmtd_response!(Unban);
    impl_into_lmtd_response!(EstimateNetworkHashesPerSecond);
    impl_into_lmtd_response!(GetMempoolEntriesByAddresses);
    impl_into_lmtd_response!(GetCoinSupply);
    impl_into_lmtd_response!(Ping);
    impl_into_lmtd_response!(GetMetrics);
    impl_into_lmtd_response!(GetConnections);
    impl_into_lmtd_response!(GetSystemInfo);
    impl_into_lmtd_response!(GetServerInfo);
    impl_into_lmtd_response!(GetSyncStatus);
    impl_into_lmtd_response!(GetDaaScoreTimestampEstimate);
    impl_into_lmtd_response!(GetFeeEstimate);
    impl_into_lmtd_response!(GetFeeEstimateExperimental);
    impl_into_lmtd_response!(GetCurrentBlockColor);
    impl_into_lmtd_response!(GetUtxoReturnAddress);

    impl_into_lmtd_notify_response!(NotifyBlockAdded);
    impl_into_lmtd_notify_response!(NotifyNewBlockTemplate);
    impl_into_lmtd_notify_response!(NotifyUtxosChanged);
    impl_into_lmtd_notify_response!(NotifyPruningPointUtxoSetOverride);
    impl_into_lmtd_notify_response!(NotifyFinalityConflict);
    impl_into_lmtd_notify_response!(NotifyVirtualDaaScoreChanged);
    impl_into_lmtd_notify_response!(NotifyVirtualChainChanged);
    impl_into_lmtd_notify_response!(NotifySinkBlueScoreChanged);

    impl_into_lmtd_notify_response!(NotifyUtxosChanged, StopNotifyingUtxosChanged);
    impl_into_lmtd_notify_response!(NotifyPruningPointUtxoSetOverride, StopNotifyingPruningPointUtxoSetOverride);

    macro_rules! impl_into_lmtd_response {
        ($name:tt) => {
            paste::paste! {
                impl_into_lmtd_response_ex!(lmt_rpc_core::[<$name Response>],[<$name ResponseMessage>],[<$name Response>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            paste::paste! {
                impl_into_lmtd_response_base!(lmt_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>],[<$protowire_name Response>]);
            }
        };
    }
    use impl_into_lmtd_response;

    macro_rules! impl_into_lmtd_response_base {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<$core_struct>> for $protowire_struct {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    item.as_ref().map_err(|x| (*x).clone()).into()
                }
            }

            impl From<RpcError> for $protowire_struct {
                fn from(item: RpcError) -> Self {
                    let x: RpcResult<&$core_struct> = Err(item);
                    x.into()
                }
            }

            impl From<$protowire_struct> for lmtd_response::Payload {
                fn from(item: $protowire_struct) -> Self {
                    lmtd_response::Payload::$variant(item)
                }
            }

            impl From<$protowire_struct> for LmtdResponse {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(lmtd_response::Payload::$variant(item)) }
                }
            }
        };
    }
    use impl_into_lmtd_response_base;

    macro_rules! impl_into_lmtd_response_ex {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<&$core_struct>> for lmtd_response::Payload {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    lmtd_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<&$core_struct>> for LmtdResponse {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<RpcResult<$core_struct>> for lmtd_response::Payload {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    lmtd_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<$core_struct>> for LmtdResponse {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl_into_lmtd_response_base!($core_struct, $protowire_struct, $variant);

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&lmtd_response::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &lmtd_response::Payload) -> RpcResult<Self> {
                    if let lmtd_response::Payload::$variant(response) = item {
                        response.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&LmtdResponse> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &LmtdResponse) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("LmtResponse".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }
        };
    }
    use impl_into_lmtd_response_ex;

    macro_rules! impl_into_lmtd_notify_response {
        ($name:tt) => {
            impl_into_lmtd_response!($name);

            paste::paste! {
                impl_into_lmtd_notify_response_ex!(lmt_rpc_core::[<$name Response>],[<$name ResponseMessage>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            impl_into_lmtd_response!($core_name, $protowire_name);

            paste::paste! {
                impl_into_lmtd_notify_response_ex!(lmt_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>]);
            }
        };
    }
    use impl_into_lmtd_notify_response;

    macro_rules! impl_into_lmtd_notify_response_ex {
        ($($core_struct:ident)::+, $protowire_struct:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl<T> From<Result<(), T>> for $protowire_struct
            where
                T: Into<RpcError>,
            {
                fn from(item: Result<(), T>) -> Self {
                    item
                        .map(|_| $($core_struct)::+{})
                        .map_err(|err| err.into()).into()
                }
            }

        };
    }
    use impl_into_lmtd_notify_response_ex;
}
