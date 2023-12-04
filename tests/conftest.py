import os
import json
import pathlib

import eth_abi
import pytest

from solana.keypair import Keypair
from eth_keys import keys as eth_keys
from solana.publickey import PublicKey
from solana.rpc.commitment import Confirmed

from .solana_utils import EvmLoader, create_treasury_pool_address, make_new_user, \
    deposit_neon, solana_client, get_solana_balance, wait_for_account_to_exists
from .utils.constants import NEON_TOKEN_MINT_ID
from .utils.contract import deploy_contract
from .utils.storage import create_holder
from .utils.types import TreasuryPool, Caller, Contract
from .utils.neon_api_client import NeonApiClient


def pytest_addoption(parser):
    parser.addoption(
        "--neon-api-uri", action="store", default="http://neon_api:8085/api",
        help=""
    )


def pytest_configure():
    if "CI" in os.environ:
        pytest.CONTRACTS_PATH = pathlib.Path("/opt/solidity")
    else:
        pytest.CONTRACTS_PATH = pathlib.Path(__file__).parent / "contracts"


@pytest.fixture(scope="session")
def evm_loader(operator_keypair: Keypair) -> EvmLoader:
    loader = EvmLoader(operator_keypair)
    return loader


def prepare_operator(key_file):
    with open(key_file, "r") as key:
        secret_key = json.load(key)[:32]
        account = Keypair.from_secret_key(secret_key)

    solana_client.request_airdrop(account.public_key, 1000 * 10 ** 9, commitment=Confirmed)
    wait_for_account_to_exists(solana_client, account.public_key)

    a = solana_client.get_account_info(account.public_key, commitment=Confirmed)
    print(f"{a}")

    operator_ether = eth_keys.PrivateKey(account.secret_key[:32]).public_key.to_canonical_address()

    evm_loader = EvmLoader(account)
    ether_balance_pubkey = evm_loader.ether2balance(operator_ether)
    acc_info = solana_client.get_account_info(ether_balance_pubkey, commitment=Confirmed)
    if acc_info.value is None:
        evm_loader.create_balance_account(operator_ether)

    return account

@pytest.fixture(scope="session")
def default_operator_keypair() -> Keypair:
    """
    Initialized solana keypair with balance. Get private keys from ci/operator-keypairs/id.json
    """
    key_path = pathlib.Path(__file__).parent.parent / "operator-keypairs"
    key_file = key_path / "id.json"
    return prepare_operator(key_file)

@pytest.fixture(scope="session")
def operator_keypair(worker_id) -> Keypair:
    """
    Initialized solana keypair with balance. Get private keys from ci/operator-keypairs
    """
    key_path = pathlib.Path(__file__).parent.parent / "operator-keypairs"
    if worker_id in ("master", "gw1"):
        key_file = key_path / "id.json"
    else:
        file_id = int(worker_id[-1]) + 2
        key_file = key_path / f"id{file_id}.json"
    return prepare_operator(key_file)


@pytest.fixture(scope="session")
def second_operator_keypair(worker_id) -> Keypair:
    """
    Initialized solana keypair with balance. Get private key from cli or ./ci/operator-keypairs
    """
    key_path = pathlib.Path(__file__).parent.parent / "operator-keypairs"
    if worker_id in ("master", "gw1"):
        key_file = key_path / "id20.json"
    else:
        file_id = 20 + int(worker_id[-1]) + 2
        key_file = key_path / f"id{file_id}.json"

    return prepare_operator(key_file)


@pytest.fixture(scope="session")
def treasury_pool(evm_loader) -> TreasuryPool:
    index = 2
    address = create_treasury_pool_address(index)
    index_buf = index.to_bytes(4, 'little')
    return TreasuryPool(index, address, index_buf)


@pytest.fixture(scope="function")
def user_account(evm_loader) -> Caller:
    return make_new_user(evm_loader)


@pytest.fixture(scope="session")
def session_user(evm_loader) -> Caller:
    return make_new_user(evm_loader)


@pytest.fixture(scope="session")
def second_session_user(evm_loader) -> Caller:
    return make_new_user(evm_loader)


@pytest.fixture(scope="session")
def sender_with_tokens(evm_loader, operator_keypair) -> Caller:
    user = make_new_user(evm_loader)
    deposit_neon(evm_loader, operator_keypair, user.eth_address, 100000)
    return user


@pytest.fixture(scope="session")
def holder_acc(operator_keypair) -> PublicKey:
    return create_holder(operator_keypair)


@pytest.fixture(scope="function")
def new_holder_acc(operator_keypair) -> PublicKey:
    return create_holder(operator_keypair)


@pytest.fixture(scope="function")
def rw_lock_contract(evm_loader: EvmLoader, operator_keypair: Keypair, session_user: Caller,
                     treasury_pool) -> Contract:
    return deploy_contract(operator_keypair, session_user, "rw_lock.binary", evm_loader, treasury_pool)


@pytest.fixture(scope="function")
def rw_lock_caller(evm_loader: EvmLoader, operator_keypair: Keypair,
                   session_user: Caller, treasury_pool: TreasuryPool, rw_lock_contract: Contract) -> Contract:
    constructor_args = eth_abi.encode(['address'], [rw_lock_contract.eth_address.hex()])
    return deploy_contract(operator_keypair, session_user, "rw_lock_caller.binary", evm_loader,
                           treasury_pool, encoded_args=constructor_args)


@pytest.fixture(scope="function")
def string_setter_contract(evm_loader: EvmLoader, operator_keypair: Keypair, session_user: Caller,
                           treasury_pool) -> Contract:
    return deploy_contract(operator_keypair, session_user, "string_setter.binary", evm_loader, treasury_pool)


@pytest.fixture(scope="session")
def calculator_contract(evm_loader: EvmLoader, operator_keypair: Keypair, session_user: Caller,
                        treasury_pool) -> Contract:
    return deploy_contract(operator_keypair, session_user, "Calculator.binary", evm_loader, treasury_pool)


@pytest.fixture(scope="session")
def calculator_caller_contract(evm_loader: EvmLoader, operator_keypair: Keypair, session_user: Caller,
                               treasury_pool, calculator_contract) -> Contract:
    constructor_args = eth_abi.encode(['address'], [calculator_contract.eth_address.hex()])

    return deploy_contract(operator_keypair, session_user, "CalculatorCaller.binary", evm_loader, treasury_pool,
                           encoded_args=constructor_args)


@pytest.fixture(scope="session")
def neon_api_client(request):
    client = NeonApiClient(url=request.config.getoption("--neon-api-uri"))
    return client
