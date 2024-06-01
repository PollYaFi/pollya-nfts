pub use crate::msg::{InstantiateMsg, QueryMsg, ExtensionMsg, ExtensionMessageType, PointsType, AddPointsMsg, Metadata, Extension, PollyaNFTInfoResponse};
use cosmwasm_std::Empty;
pub use cw721_base::{
    entry::{execute as _execute, query as _query},
    ContractError, Cw721Contract, InstantiateMsg as Cw721BaseInstantiateMsg,
    MinterResponse,
};
use cw721_base::{msg::{QueryMsg as Cw721QueryMsg}};
use cw721::{NftInfoResponse};
pub use crate::state::{PollyaAttributes};
use serde_json_wasm;
use sha2::{Sha256, Digest};

pub mod msg;
pub mod query;
pub mod state;
pub mod helpers;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-non-transferable";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Cw721NonTransferableContract<'a> = Cw721Contract<'a, Extension, Empty, ExtensionMsg, Empty>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, ExtensionMsg>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;
    use crate::helpers::from_base64;
    use crate::msg::{ChangeAdminMsg, FinalizeChangeAdminMsg};
    use crate::query::admin;
    use crate::state::{Config, CONFIG, PollyaAttributes, POLLYA_LOYALTY_POINTS};
    use cosmwasm_std::{
        entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, from_binary, StdError
    };
    
    #[entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let admin_addr: Option<Addr> = msg
        .admin
        .as_deref()
        .map(|s| deps.api.addr_validate(s))
        .transpose()?;

        let admin_pub_key = msg.admin_pub_key.clone();
        
        let config = Config { admin: admin_addr, admin_pub_key: admin_pub_key, transition_admin_pub_key: None };
        
        CONFIG.save(deps.storage, &config)?;
        
        let cw721_base_instantiate_msg = Cw721BaseInstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            minter: msg.minter,
        };
        
        Cw721NonTransferableContract::default().instantiate(
            deps.branch(),
            env,
            info,
            cw721_base_instantiate_msg,
        )?;
        
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        
        Ok(Response::default()
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
    }
    
    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, cw721_base::ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let admin = config.admin.clone();
        match admin {
            Some(admin) => {
                if admin == info.sender {
                    match msg {
                        ExecuteMsg::Extension {
                            msg
                        } => {
                            execute_extension_msg(deps, env, info, &config, &msg)?;
                            Ok(Response::default())
                        },
                        _ => Cw721NonTransferableContract::default().execute(deps, env, info, msg)
                    }
                } else {
                    match msg {
                        ExecuteMsg::Extension {
                            msg
                        } => {
                            execute_extension_msg(deps, env, info, &config, &msg)?;
                            Ok(Response::default())
                        },
                        _ => Err(ContractError::Ownership(
                            cw721_base::OwnershipError::NotOwner,
                        ))
                    }
                }
            }
            None => match msg {
                ExecuteMsg::Mint {
                    token_id,
                    owner,
                    token_uri,
                    extension,
                } => {
                    Cw721NonTransferableContract::default()
                        .mint(deps, info, token_id.clone(), owner, token_uri, extension)
                },
                ExecuteMsg::Extension {
                    msg
                } => {  
                    execute_extension_msg(deps, env, info, &config, &msg)?;
                    Ok(Response::default())
                },
                _ => Err(ContractError::Ownership(
                    cw721_base::OwnershipError::NotOwner,
                )),
            },
        }
    }
    
    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        let _msg = &msg;
        match _msg {
            QueryMsg::Admin {} => to_binary(&admin(deps)?),
            QueryMsg::PollyaNftInfo {
                token_id
            } => {
                let attributes = POLLYA_LOYALTY_POINTS.load(deps.storage, token_id.as_str()).unwrap_or_default();
                let q_msg = Cw721QueryMsg::NftInfo { token_id: token_id.to_string() };
                let result = Cw721NonTransferableContract::default().query(deps, env, q_msg).unwrap();
                let nft_value: NftInfoResponse<Extension> = from_binary(&result).unwrap();
                let response: PollyaNFTInfoResponse = PollyaNFTInfoResponse{
                    nft: nft_value,
                    pollya_attributes: attributes
                };
                to_binary(&response)
            },
            _ => _query(deps, env, msg.into()),
        }
    }
    
    fn execute_extension_msg(deps: DepsMut,
        env: Env,
        info: MessageInfo,
        config: &Config,
        msg: &ExtensionMsg,
    ) -> Result<bool, StdError> {
        match msg.message_type {
            ExtensionMessageType::AddPoints => {
                let add_points_msg: AddPointsMsg = serde_json_wasm::from_str(msg.value.as_str()).unwrap();
                let signature = msg.signature.clone();
                let signed_by = msg.signed_by.clone();
                add_points(deps, info, env, config, msg.token_id.clone().unwrap(), &add_points_msg, signature.unwrap(), signed_by.unwrap())
            },
            ExtensionMessageType::ChangeAdmin => {
                change_admin(deps, info, env, &msg.value)
            },
            ExtensionMessageType::FinalizeChangeAdmin => {
                finalize_change_admin(deps, info, env, &msg.value)
            },
            _ => Ok(true)
        }
    }

    fn change_admin(deps: DepsMut, info: MessageInfo, env: Env, msg: &String) -> Result<bool, StdError> {
        let ownership = cw_ownable::get_ownership(deps.storage).unwrap();
        assert!(ownership.owner.unwrap() == info.sender);
        let change_admin_msg: ChangeAdminMsg = serde_json_wasm::from_str(msg).unwrap();
        let mut config = CONFIG.load(deps.storage).unwrap();
        config.transition_admin_pub_key = Some(change_admin_msg.new_admin_public_key);
        CONFIG.save(deps.storage, &config)?;
        let result = Cw721NonTransferableContract::update_ownership(deps, env, info, change_admin_msg.action);
        if result.is_ok() {
            Ok(true)
        } else {
            Err(StdError::GenericErr { msg: format!("{:?}", result.err())})
        }
    }

    fn finalize_change_admin(deps: DepsMut, info: MessageInfo, env: Env, msg: &String) -> Result<bool, StdError> {
        let owner = cw_ownable::get_ownership(deps.storage).unwrap();
        assert!(owner.pending_owner.unwrap() == info.sender);
        let finalize_admin_msg: FinalizeChangeAdminMsg = serde_json_wasm::from_str(msg).unwrap();
        assert!(finalize_admin_msg.action == cw_ownable::Action::AcceptOwnership);
        let mut config = CONFIG.load(deps.storage).unwrap();
        config.admin_pub_key = config.transition_admin_pub_key;
        config.admin = Some(info.sender.clone());
        config.transition_admin_pub_key = None;
        CONFIG.save(deps.storage, &config)?;
        let result = Cw721NonTransferableContract::update_ownership(deps, env, info, finalize_admin_msg.action);
        if result.is_ok() {
            Ok(true)
        } else {
            Err(StdError::GenericErr { msg: format!("{:?}", result.err())})
        }
    }
    
    fn add_points(deps: DepsMut, info: MessageInfo, env: Env, config: &Config, token_id: String, add_points_msg: &AddPointsMsg, signature: String, signed_by: String) -> Result<bool, StdError> {
        let pollya_points = POLLYA_LOYALTY_POINTS.load(deps.storage, token_id.as_str()).unwrap_or_default();
        let base_msg = format!(
            "{}{}{}{}",
            info.sender.as_str(),
            env.contract.address.as_str(),
            token_id.clone(),
            add_points_msg.points
        );
        let (updated_points, message) = match add_points_msg.points_type {
            PointsType::PollCreated => {
                (
                    PollyaAttributes {
                        total_points: pollya_points.total_points + add_points_msg.points,
                        poll_creation_points: pollya_points.poll_creation_points + add_points_msg.points,
                        poll_vote_points: pollya_points.poll_vote_points
                    },
                    format!(
                        "{}{}", 
                        "setup",
                        base_msg
                    )
                )
            },
            PointsType::PollVoted => {
                (PollyaAttributes {
                    total_points: pollya_points.total_points + add_points_msg.points,
                    poll_creation_points: pollya_points.poll_creation_points,
                    poll_vote_points: pollya_points.poll_vote_points + add_points_msg.points,
                },
                format!("{}{}", "vote", base_msg))
            }
        };
        let mut sha256 = Sha256::default();
        sha256.update(message.as_bytes());
        let hash: [u8; 32] = sha256.finalize().into();
        let sig = from_base64(&signature);
        let mut pub_key = config.admin_pub_key.clone();
        if pub_key.clone().unwrap() != signed_by {
            pub_key = config.transition_admin_pub_key.clone();
            if pub_key.clone().unwrap() != signed_by {
                return Err(StdError::GenericErr { msg: format!("Unexpected signer public key")})
            }
        }
        let public_key = from_base64(&pub_key.unwrap());
        let sig_verified = deps.api.secp256k1_verify(&hash, sig.as_slice(), public_key.as_slice());
        if sig_verified.is_ok() {
            POLLYA_LOYALTY_POINTS.save(deps.storage, token_id.as_str(), &updated_points)?;
            Ok(true)
        } else {
            Err(StdError::GenericErr { msg: format!("{} {:?}, {:?}", sig_verified.err().unwrap(), hex::encode(hash), hex::encode(public_key))})
        }
    }
}
