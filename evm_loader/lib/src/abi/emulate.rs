use super::params_to_neon_error;
use crate::commands::emulate::{self, EmulateResponse};
use crate::commands::get_config::BuildConfigSimulator;
use crate::rpc::Rpc;
use crate::Config;
use crate::{types::EmulateApiRequest, NeonResult};

pub async fn execute(
    rpc: &(impl Rpc + BuildConfigSimulator),
    config: &Config,
    params: &str,
) -> NeonResult<EmulateResponse> {
    let params: EmulateApiRequest =
        serde_json::from_str(params).map_err(|_| params_to_neon_error(params))?;

    emulate::execute(rpc, config.evm_loader, params.body, None).await
}
