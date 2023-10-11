mod abi;
pub mod account_storage;
pub mod build_info;
pub mod build_info_common;
pub mod commands;
pub mod config;
pub mod errors;
pub mod rpc;
pub mod syscall_stubs;
pub mod tracing;
pub mod types;

use abi::_MODULE_WM_;
use abi_stable::export_root_module;
pub use config::Config;
pub use errors::NeonError;
use neon_lib_interface::NeonEVMLib_Ref;

pub type NeonResult<T> = Result<T, NeonError>;

const MODULE: NeonEVMLib_Ref = NeonEVMLib_Ref(_MODULE_WM_.static_as_prefix());

#[export_root_module]
pub fn get_root_module() -> NeonEVMLib_Ref {
    MODULE
}

use strum_macros::{AsRefStr, Display, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString, IntoStaticStr, AsRefStr)]
pub enum LibMethods {
    #[strum(serialize = "emulate")]
    Emulate,
    #[strum(serialize = "get_storage_at")]
    GetStorageAt,
    #[strum(serialize = "config")]
    GetConfig,
    #[strum(serialize = "balance")]
    GetBalance,
    #[strum(serialize = "contract")]
    GetContract,
    #[strum(serialize = "holder")]
    GetHolder,
    #[strum(serialize = "trace")]
    Trace,
    #[strum(serialize = "cancel_trx")]
    CancelTrx,
    #[strum(serialize = "collect_treasury")]
    CollectTreasury,
    #[strum(serialize = "get_neon_elf")]
    GetNeonElf,
    #[strum(serialize = "init_environment")]
    InitEnvironment,
}
