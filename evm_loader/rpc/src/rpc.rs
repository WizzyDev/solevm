use crate::context::Context;
use crate::error::NeonRPCError;
use crate::handlers::{
    emulate, get_balance, get_config, get_contract, get_holder, get_storage_at, info, trace,
};

use jsonrpc_v2::{Data, MapRouter, Server};
use neon_lib::LibMethods;
use std::sync::Arc;

pub fn build_rpc(ctx: Context) -> Result<Arc<Server<MapRouter>>, NeonRPCError> {
    let mut rpc_builder = Server::new().with_data(Data::new(ctx));

    rpc_builder = rpc_builder.with_method("build_info", info::handle);

    rpc_builder =
        rpc_builder.with_method(LibMethods::GetStorageAt.to_string(), get_storage_at::handle);
    rpc_builder = rpc_builder.with_method(LibMethods::Trace.to_string(), trace::handle);
    rpc_builder = rpc_builder.with_method(LibMethods::Emulate.to_string(), emulate::handle);
    rpc_builder = rpc_builder.with_method(LibMethods::GetBalance.to_string(), get_balance::handle);
    rpc_builder = rpc_builder.with_method(LibMethods::GetConfig.to_string(), get_config::handle);
    rpc_builder = rpc_builder.with_method(LibMethods::GetHolder.to_string(), get_holder::handle);
    rpc_builder =
        rpc_builder.with_method(LibMethods::GetContract.to_string(), get_contract::handle);

    let rpc = rpc_builder.finish();

    Ok(rpc)
}
