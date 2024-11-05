use crate::{
    chain::{ChainUid, CrossChainUser, CrossChainUserWithLimit},
    fee::{DenomFees, PartnerFee},
    liquidity::{AddLiquidityRequest, RemoveLiquidityRequest},
    swap::{NextSwapPair, SwapRequest},
    token::{Pair, PairWithDenom, PairWithDenomAndAmount, Token, TokenType, TokenWithDenom},
    utils::pagination::Pagination,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, IbcPacketAckMsg, IbcPacketReceiveMsg, Uint128};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    // Router contract on VLP
    pub router_contract: String,
    pub chain_uid: ChainUid,
    pub escrow_code_id: u64,
    pub cw20_code_id: u64,
    pub is_native: bool,
}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Adds liquidity to a pool by providing tokens
    ///
    /// * `pair_info` - Information about the token pair and amounts to add
    /// * `slippage_tolerance_bps` - Maximum allowed slippage in basis points (1/10000)
    /// * `timeout` - Optional timeout in seconds for the transaction
    AddLiquidityRequest {
        pair_info: PairWithDenomAndAmount,
        slippage_tolerance_bps: u64,
        timeout: Option<u64>,
    },

    /// Executes a swap between tokens
    ///
    /// * `asset_in` - Token being swapped from
    /// * `amount_in` - Amount of input token to swap
    /// * `asset_out` - Token being swapped to
    /// * `min_amount_out` - Minimum amount of output token to receive
    /// * `timeout` - Optional timeout in seconds for the transaction
    /// * `swaps` - Vector of intermediate swaps to perform
    /// * `cross_chain_addresses` - Prioritized list of cross-chain addresses to receive tokens
    /// * `partner_fee` - Optional partner fee configuration
    ExecuteSwapRequest {
        asset_in: TokenWithDenom,
        amount_in: Uint128,
        asset_out: Token,
        min_amount_out: Uint128,
        timeout: Option<u64>,
        swaps: Vec<NextSwapPair>,
        // First element in array has highest priority
        cross_chain_addresses: Vec<CrossChainUserWithLimit>,
        partner_fee: Option<PartnerFee>,
    },

    /// Registers a new token denomination
    ///
    /// * `token` - Token with denomination to register
    RequestRegisterDenom { token: TokenWithDenom },

    /// Deregisters an existing token denomination
    ///
    /// * `token` - Token with denomination to deregister
    RequestDeregisterDenom { token: TokenWithDenom },

    /// Requests creation of a new liquidity pool
    ///
    /// * `pair` - Token pair with denominations for the pool
    /// * `timeout` - Optional timeout in seconds for the transaction
    /// * `lp_token_name` - Name for the LP token
    /// * `lp_token_symbol` - Symbol for the LP token
    /// * `lp_token_decimal` - Decimal places for the LP token
    /// * `lp_token_marketing` - Optional marketing info for the LP token
    RequestPoolCreation {
        pair: PairWithDenom,
        timeout: Option<u64>,
        lp_token_name: String,
        lp_token_symbol: String,
        lp_token_decimal: u8,
        lp_token_marketing: Option<cw20_base::msg::InstantiateMarketingInfo>,
    },

    /// Requests registration of an escrow for a token
    ///
    /// * `token` - Token with denomination to register escrow for
    /// * `timeout` - Optional timeout in seconds for the transaction
    RequestRegisterEscrow {
        token: TokenWithDenom,
        timeout: Option<u64>,
    },

    /// Updates the IBC channel used for hub communication
    ///
    /// * `new_channel` - New channel ID to use
    UpdateHubChannel { new_channel: String },

    /// Withdraws tokens from virtual balance
    ///
    /// * `token` - Token to withdraw
    /// * `amount` - Amount to withdraw
    /// * `cross_chain_addresses` - Prioritized list of cross-chain addresses to receive tokens
    /// * `timeout` - Optional timeout in seconds for the transaction
    WithdrawVirtualBalance {
        token: Token,
        amount: Uint128,
        cross_chain_addresses: Vec<CrossChainUserWithLimit>,
        timeout: Option<u64>,
    },

    /// Transfers virtual balance between users
    ///
    /// * `token` - Token to transfer
    /// * `amount` - Amount to transfer
    /// * `recipient_address` - Cross-chain address of recipient
    /// * `timeout` - Optional timeout in seconds for the transaction
    TransferVirtualBalance {
        token: Token,
        amount: Uint128,
        recipient_address: CrossChainUser,
        timeout: Option<u64>,
    },

    /// Deposits tokens into the contract
    ///
    /// * `asset_in` - Token with denomination to deposit
    /// * `amount_in` - Amount to deposit
    /// * `timeout` - Optional timeout in seconds for the transaction
    /// * `recipient` - Optional cross-chain recipient address
    DepositToken {
        asset_in: TokenWithDenom,
        amount_in: Uint128,
        timeout: Option<u64>,
        recipient: Option<CrossChainUser>,
    },

    /// Updates factory contract state parameters
    ///
    /// * `router_contract` - Optional new router contract address
    /// * `admin` - Optional new admin address
    /// * `escrow_code_id` - Optional new escrow contract code ID
    /// * `cw20_code_id` - Optional new CW20 contract code ID
    /// * `is_native` - Optional flag for native token support
    UpdateFactoryState {
        router_contract: Option<String>,
        admin: Option<String>,
        escrow_code_id: Option<u64>,
        cw20_code_id: Option<u64>,
        is_native: Option<bool>,
    },

    /// Receives CW20 tokens
    ///
    /// * `cw20_msg` - CW20 receive message
    Receive(Cw20ReceiveMsg),

    /// Handles IBC acknowledgment and timeout callbacks
    ///
    /// * `ack` - IBC packet acknowledgment message
    IbcCallbackAckAndTimeout { ack: IbcPacketAckMsg },

    /// Handles IBC receive callbacks
    ///
    /// * `receive_msg` - IBC packet receive message
    IbcCallbackReceive { receive_msg: IbcPacketReceiveMsg },

    /// Handles native token receive callbacks
    ///
    /// * `msg` - Callback message in binary format
    NativeReceiveCallback { msg: Binary },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetVlpResponse)]
    GetVlp { pair: Pair },

    #[returns(GetLPTokenResponse)]
    GetLPToken { vlp: String },

    #[returns(StateResponse)]
    GetState {},

    #[returns(PartnerFeesCollectedResponse)]
    GetPartnerFeesCollected {},

    // Query to get all pools in the factory
    #[returns(AllPoolsResponse)]
    GetAllPools {},

    // Query to get all pools in the factory
    #[returns(AllTokensResponse)]
    GetAllTokens {},

    // Fetch pending swaps with pagination for a user
    #[returns(GetPendingSwapsResponse)]
    PendingSwapsUser {
        user: Addr,
        pagination: Pagination<Uint128>,
    },
    #[returns(GetPendingLiquidityResponse)]
    PendingLiquidity {
        user: Addr,
        pagination: Pagination<Uint128>,
    },
    #[returns(GetPendingRemoveLiquidityResponse)]
    PendingRemoveLiquidity {
        user: Addr,
        pagination: Pagination<Uint128>,
    },

    #[returns(GetEscrowResponse)]
    GetEscrow { token_id: String },
}

#[cw_serde]
pub struct GetVlpResponse {
    pub vlp_address: String,
}

#[cw_serde]
pub struct GetLPTokenResponse {
    pub token_address: Addr,
}

#[cw_serde]
pub struct GetEscrowResponse {
    pub escrow_address: Option<Addr>,
    pub denoms: Vec<TokenType>,
}
// We define a custom struct for each query response
#[cw_serde]
pub struct StateResponse {
    pub chain_uid: ChainUid,
    pub router_contract: String,
    pub hub_channel: Option<String>,
    pub admin: String,
    // Escrow Code ID
    pub escrow_code_id: u64,
    // CW20 Code ID
    pub cw20_code_id: u64,
    pub is_native: bool,
    pub partner_fees_collected: DenomFees,
}

#[cw_serde]
pub struct PartnerFeesCollectedResponse {
    pub total: DenomFees,
}

#[cw_serde]
pub struct PartnerFeesCollectedPerDenomResponse {
    pub total: Uint128,
}

#[cw_serde]
pub struct AllPoolsResponse {
    pub pools: Vec<PoolVlpResponse>, // Assuming pool addresses are strings
}
#[cw_serde]
pub struct PoolVlpResponse {
    pub pair: Pair,
    pub vlp: String,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct RegisterFactoryResponse {
    pub factory_address: String,
    pub chain_id: String,
}

#[cw_serde]
pub struct ReleaseEscrowResponse {
    pub factory_address: String,
    pub chain_id: String,
    pub amount: Uint128,
    pub token: Token,
    pub to_address: String,
}

#[cw_serde]
pub struct GetPendingSwapsResponse {
    pub pending_swaps: Vec<SwapRequest>,
}
#[cw_serde]
pub struct GetPendingLiquidityResponse {
    pub pending_add_liquidity: Vec<AddLiquidityRequest>,
}

#[cw_serde]
pub struct GetPendingRemoveLiquidityResponse {
    pub pending_remove_liquidity: Vec<RemoveLiquidityRequest>,
}

#[cw_serde]
pub struct AllTokensResponse {
    pub tokens: Vec<Token>, // Assuming pool addresses are strings
}
