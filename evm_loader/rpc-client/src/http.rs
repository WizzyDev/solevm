use async_trait::async_trait;
use jsonrpsee_core::{client::ClientT, rpc_params};
use jsonrpsee_http_client::{HttpClient, HttpClientBuilder};
use neon_lib::LibMethods;
use neon_lib::{
    commands::{
        emulate::EmulateResponse, get_balance::GetBalanceResponse, get_config::GetConfigResponse,
        get_contract::GetContractResponse, get_holder::GetHolderResponse,
        get_storage_at::GetStorageAtReturn,
    },
    types::{
        EmulateApiRequest, GetBalanceRequest, GetContractRequest, GetHolderRequest,
        GetStorageAtRequest,
    },
};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{config::NeonRpcClientConfig, NeonRpcClient, NeonRpcClientResult};

pub struct NeonRpcHttpClient {
    client: HttpClient,
}

impl NeonRpcHttpClient {
    pub async fn new(config: NeonRpcClientConfig) -> NeonRpcClientResult<NeonRpcHttpClient> {
        Ok(NeonRpcHttpClient {
            client: HttpClientBuilder::default().build(config.url)?,
        })
    }
}

pub struct NeonRpcHttpClientBuilder {}

impl NeonRpcHttpClientBuilder {
    pub fn new() -> NeonRpcHttpClientBuilder {
        NeonRpcHttpClientBuilder {}
    }

    pub async fn build(&self, url: impl Into<String>) -> NeonRpcClientResult<NeonRpcHttpClient> {
        let config = NeonRpcClientConfig::new(url);
        NeonRpcHttpClient::new(config).await
    }
}

impl Default for NeonRpcHttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl NeonRpcClient for NeonRpcHttpClient {
    async fn emulate(&self, params: EmulateApiRequest) -> NeonRpcClientResult<EmulateResponse> {
        self.request(LibMethods::Emulate, params).await
    }

    async fn balance(
        &self,
        params: GetBalanceRequest,
    ) -> NeonRpcClientResult<Vec<GetBalanceResponse>> {
        self.request(LibMethods::GetBalance, params).await
    }

    async fn get_contract(
        &self,
        params: GetContractRequest,
    ) -> NeonRpcClientResult<Vec<GetContractResponse>> {
        self.request(LibMethods::GetContract, params).await
    }

    async fn get_config(&self) -> NeonRpcClientResult<GetConfigResponse> {
        self.request_without_params(LibMethods::GetConfig).await
    }

    async fn get_holder(&self, params: GetHolderRequest) -> NeonRpcClientResult<GetHolderResponse> {
        self.request(LibMethods::GetHolder, params).await
    }

    async fn get_storage_at(
        &self,
        params: GetStorageAtRequest,
    ) -> NeonRpcClientResult<GetStorageAtReturn> {
        self.request(LibMethods::GetStorageAt, params).await
    }

    async fn trace(&self, params: EmulateApiRequest) -> NeonRpcClientResult<serde_json::Value> {
        self.request(LibMethods::Trace, params).await
    }
}

impl NeonRpcHttpClient {
    async fn request<R, P>(&self, method: LibMethods, params: P) -> NeonRpcClientResult<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        Ok(self
            .client
            .request(method.into(), rpc_params![params])
            .await?)
    }

    async fn request_without_params<R>(&self, method: LibMethods) -> NeonRpcClientResult<R>
    where
        R: DeserializeOwned,
    {
        Ok(self.client.request(method.into(), rpc_params![]).await?)
    }
}
