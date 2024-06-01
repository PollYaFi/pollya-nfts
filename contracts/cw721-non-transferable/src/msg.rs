use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Empty, CustomMsg};
use cw721_base::{msg::{QueryMsg as Cw721QueryMsg}};
use cw721_base;
use cw721::{NftInfoResponse};
use crate::state::PollyaAttributes;
use thiserror::Error;


#[derive(Error, Debug)]
pub struct NFTError {
    message: String
}

impl fmt::Display for NFTError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
     write!(f, "{}", self.message)
   }
}

#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub data: Option<String>,
}

pub type Extension = Option<Metadata>;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub admin_pub_key: Option<String>,
    pub name: String,
    pub symbol: String,
    pub minter: String,
}

#[cw_serde]
pub enum QueryMsg {
    Admin {},
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NumTokens {},
    ContractInfo {},
    NftInfo {
        token_id: String,
    },
    PollyaNftInfo {
        token_id: String,
    },
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Minter {},
}

impl From<QueryMsg> for Cw721QueryMsg<Empty> {
    fn from(msg: QueryMsg) -> Cw721QueryMsg<Empty> {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Cw721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::NumTokens {} => Cw721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => Cw721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => Cw721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Cw721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Cw721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::Minter {} => Cw721QueryMsg::Minter {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

#[cw_serde]
pub enum ExtensionMessageType {
    AddPoints,
    ChangeAdmin,
    FinalizeChangeAdmin
}

#[cw_serde]
pub enum PointsType {
    PollCreated,
    PollVoted
}

#[cw_serde]
pub struct AddPointsMsg {
    pub points: u128,
    pub points_type: PointsType
}

#[cw_serde]
pub struct ChangeAdminMsg {
    pub new_admin_public_key: String,
    pub action: cw_ownable::Action
}

#[cw_serde]
pub struct FinalizeChangeAdminMsg {
    pub action: cw_ownable::Action
}




#[cw_serde]
pub struct ExtensionMsg {
    pub message_type: ExtensionMessageType,
    pub token_id: Option<String>,
    pub value: String,
    pub signature: Option<String>,
    pub signed_by: Option<String>
}

impl CustomMsg for ExtensionMsg {
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: Option<String>,
}


#[cw_serde]
pub struct PollyaNFTInfoResponse {
    pub nft: NftInfoResponse<Extension>,
    pub pollya_attributes: PollyaAttributes
}
