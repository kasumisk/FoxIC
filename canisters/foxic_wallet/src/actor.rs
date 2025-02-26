use crate::http::{err404, index};
use crate::state::{owner_guard, CANISTER_OWNER, CONF};
use crate::types::{
    vec_to_u8_32, Eth, FoxICWallet, HttpRequest, HttpResponse, RPCRequest, SendArgs,
    SendArgsSimple, TransferArgs,
};
use candid::candid_method;
use ic_cdk::api::call::CallResult;
use ic_cdk::{call, caller, id, trap};
use ic_cdk_macros::*;
use ic_ledger_types::{AccountBalanceArgs, AccountIdentifier, BlockIndex, Subaccount, Tokens};
use ic_types::Principal;
use std::ops::Deref;

#[init]
#[candid_method(init)]
fn init() {
    CANISTER_OWNER.with(|o| o.replace(caller()));
}

#[query(name = "greeting")]
#[candid_method(query, rename = "greeting")]
pub fn greeting(greet: String) -> String {
    format!("hello back from rust: {}", greet.as_str()).to_string()
}

#[update(name = "set_conf", guard = "owner_guard")]
#[candid_method(update, rename = "set_conf")]
pub fn set_conf(conf: FoxICWallet) {
    CONF.with(|c| c.replace(conf));
}

#[update(name = "set_owner", guard = "owner_guard")]
#[candid_method(update, rename = "set_owner")]
pub fn set_owner(owner: Principal) {
    CANISTER_OWNER.with(|o| o.replace(owner));
}

#[query(name = "is_owner")]
#[candid_method(query, rename = "is_owner")]
pub fn is_owner() -> bool {
    CANISTER_OWNER.with(|o| o.borrow().to_text() == caller().to_text())
}

#[query(name = "get_owner")]
#[candid_method(query, rename = "get_owner")]
pub fn get_owner() -> String {
    CANISTER_OWNER.with(|o| o.borrow().to_text())
}

#[query(name = "wallet_url_get")]
#[candid_method(query, rename = "wallet_url_get")]
pub fn wallet_url_get(address: String) -> String {
    (format!("https://{}.ic0.app/address/", id().to_text()).to_string()
        + format!("{}", address.as_str()).as_str())
    .to_string()
}

/// async function getting balance
#[update(name = "wallet_balance_get")]
#[candid_method(update, rename = "wallet_balance_get")]
pub async fn wallet_balance_get(args: Option<AccountBalanceArgs>) -> Tokens {
    let wallet = CONF.with(|c| c.borrow().deref().clone());
    wallet
        .balance_of(args)
        .await
        .map_or_else(|(_, e)| trap(e.as_str()), |r| r)
}

/// async function getting account id
#[query(name = "wallet_address_get")]
#[candid_method(query, rename = "wallet_address_get")]
pub fn wallet_address_get(sub_account: Option<Vec<u8>>) -> String {
    if sub_account.is_some() {
        trap("subaccount is not correct")
    }
    CONF.with(|c| {
        c.borrow()
            .get_address(sub_account.map(|e| Subaccount(vec_to_u8_32(e))))
    })
}

/// async function getting account id
#[update(name = "wallet_icp_transfer", guard = "owner_guard")]
#[candid_method(update, rename = "wallet_icp_transfer")]
pub async fn wallet_icp_transfer(args: TransferArgs) -> Result<BlockIndex, String> {
    let wallet = CONF.with(|c| c.borrow().clone());
    wallet.transfer(args).await
}

/// async function getting account id
#[update(name = "wallet_icp_send", guard = "owner_guard")]
#[candid_method(update, rename = "wallet_icp_send")]
pub async fn wallet_icp_send(args: SendArgsSimple) -> Result<BlockIndex, String> {
    let wallet = CONF.with(|c| c.borrow().clone());
    let v8: [u8; 32] = hex::decode(args.account_id)
        .map_or_else(|e| trap("account_id is not valid hex"), |f| vec_to_u8_32(f));
    wallet
        .send(SendArgs {
            amount: args.amount,
            account_id: AccountIdentifier::try_from(v8).unwrap(),
        })
        .await
}

#[query(name = "http_request")]
#[candid_method(query, rename = "http_request")]
async fn http_request(req: HttpRequest) -> HttpResponse {
    dispatch(req).await
}

async fn dispatch(req: HttpRequest) -> HttpResponse {
    let uri = req.url.clone();
    let eth = Eth::default();
    match uri.strip_prefix("/address/") {
        Some(token) => {
            let json: RPCRequest = serde_json::from_slice(req.body.as_slice()).unwrap();
            ic_cdk::println!("{}", json.method.to_string());
            ic_cdk::println!(
                "{}",
                serde_json::to_string(&json.clone().params.unwrap()).unwrap()
            );
            match json.clone().method.as_str() {
                "eth_chainId" => Eth::chain_id(&eth, json.clone()),
                "eth_blockNumber" => Eth::block_number(&eth, json.clone()),
                "eth_getBlockByNumber" => Eth::block_by_number(&eth, json.clone()),
                "eth_getBalance" => Eth::balance(&eth, token, json.clone()).await,
                "eth_gasPrice" => Eth::gas_price(&eth, json.clone()),
                "net_version" => Eth::net_version(&eth, json.clone()),
                "eth_estimateGas" => Eth::estimate_gas(&eth, json.clone()),
                "eth_getTransactionCount" => Eth::transaction_count(&eth, json.clone()),
                _ => err404(req),
            }
        }
        None => {
            if req.url == "/" {
                index(req)
            } else {
                err404(req)
            }
        }
    }
}
