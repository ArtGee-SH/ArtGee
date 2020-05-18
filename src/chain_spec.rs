use std::collections::HashMap;
use std::convert::TryInto;

use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_json as json;

#[allow(unused_imports)]
use runtime::constants::{currency::*, time::*};
use runtime::types::*;
use runtime::{
    self, rio_assets, Block, GenesisConfig, RioAssetsConfig, RioDappDclConfig, RioGatewayConfig,
    RioGatewayEthConfig, RioPaymentFeeConfig, SessionConfig, SessionKeys, SudoConfig, SystemConfig,
    WASM_BINARY,
};
use sc_chain_spec::ChainSpecExtension;
use sc_network::config::MultiaddrWithPeerId;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::str::FromStr;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// The Rio network.
    Testnet,
    /// Titan network
    Beta,
}

pub struct SpecInfo {
    name: &'static str,
    id: &'static str,
    chain_type: ChainType,
    protocol_id: Option<&'static str>,
    pub properties: Option<sc_service::Properties>,
}

lazy_static::lazy_static! {
    pub static ref CHAIN_TYPE: HashMap<Alternative, SpecInfo> = {
        let mut m = HashMap::new();
        // value is (name, id, properties)
        m.insert(
            Alternative::Development,
            SpecInfo {
                name: "Development",
                id: "dev",
                chain_type: ChainType::Development,
                protocol_id: Some("rio_dev"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
            });
        m.insert(
            Alternative::LocalTestnet,
            SpecInfo {
                name: "Local Testnet",
                id: "local_testnet",
                chain_type: ChainType::Local,
                protocol_id: Some("rio_local"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
            });
        m.insert(
            Alternative::Testnet,
            SpecInfo {
                name: "Rio Defi Testnet",
                id: "moniker",
                chain_type: ChainType::Custom("rio testnet".to_string()),
                protocol_id: Some("rio_moniker"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
            });
        m.insert(
            Alternative::Beta,
            SpecInfo {
                name: "Rio Beta",
                id: "beta",
                chain_type: ChainType::Live,
                protocol_id: Some("rio_beta"),
                properties: Some(json::from_str(DEFAULT_PROPERTIES_MAINNET).unwrap()),
            });
        m
    };
}

const DEFAULT_PROPERTIES_MAINNET: &str = r#"
{
"tokenSymbol": "RFUEL",
"tokenDecimals": 8,
"ss58Format": 241
}
"#;

const DEFAULT_PROPERTIES_TESTNET: &str = r#"
{
"tokenSymbol": "RFUEL",
"tokenDecimals": 8,
"ss58Format": 221
}
"#;

pub fn get_alternative_from_id(id: &str) -> Result<Alternative, String> {
    for (k, s) in CHAIN_TYPE.iter() {
        if s.id == id {
            return Ok(*k);
        }
    }
    let ids = CHAIN_TYPE.iter().map(|(_, s)| s.id).collect::<Vec<_>>();
    Err(format!(
        "no support id in current `Alternative`:{:}|current support:{:?}",
        id, ids
    ))
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(seed),
        get_from_seed::<GrandpaId>(seed),
    )
}

fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        let spec: &SpecInfo = CHAIN_TYPE
            .get(&self)
            .ok_or("not support for this Alternative")?;
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                spec.name,
                spec.id,
                spec.chain_type.clone(),
                || {
                    testnet_genesis(
                        vec![get_authority_keys_from_seed("Alice")],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![(get_account_id_from_seed::<sr25519::Public>("Alice"), 10000*100000000)],
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        ],
                    )
                },
                vec![],
                None,
                spec.protocol_id,
                spec.properties.clone(),
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                spec.name,
                spec.id,
                spec.chain_type.clone(),
                || {
                    testnet_genesis(
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                        ],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            (get_account_id_from_seed::<sr25519::Public>("Alice"), 10000*100000000),
                            (get_account_id_from_seed::<sr25519::Public>("Bob"), 10000*100000000)
                        ],
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie"),
                            get_account_id_from_seed::<sr25519::Public>("Dave"),
                            get_account_id_from_seed::<sr25519::Public>("Eve"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                        ],
                    )
                },
                vec![],
                None,
                spec.protocol_id,
                spec.properties.clone(),
                None,
            ),
            Alternative::Testnet => ChainSpec::from_genesis(
                spec.name,
                spec.id,
                spec.chain_type.clone(),
                || {
                    testnet_genesis(
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                        ],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            (get_account_id_from_seed::<sr25519::Public>("Alice"), 10000*100000000),
                            (get_account_id_from_seed::<sr25519::Public>("Bob"), 10000*100000000)
                        ],
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie"),
                            get_account_id_from_seed::<sr25519::Public>("Dave"),
                            get_account_id_from_seed::<sr25519::Public>("Eve"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                        ],
                    )
                },
                vec![
                    MultiaddrWithPeerId::from_str("/ip4/47.52.149.251/tcp/30101/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR").unwrap()
                ],
                Some(
                    TelemetryEndpoints::new(
                        vec![
                            ("https://stats.staging.riodefi.com".to_string(), 0)
                        ]
                    ).unwrap()
                ),
                spec.protocol_id,
                spec.properties.clone(),
                None,
            ),
            Alternative::Beta => ChainSpec::from_genesis(
                spec.name,
                spec.id,
                spec.chain_type.clone(),
                || {
                    beta_genesis(
                        // initial_authorities
                        vec![(
                            // 5GRNaQjxizf2NtrJ7CA2EpqtHUQ6RK3iTnn1LGRoWCsEAryn
                            hex!["c0b42f17dbaf6abcb8189e6fa471f7aa805151668a972127d915c0ad23fbbe30"].unchecked_into(),
                            // 5G6FhGVzzik1hepRSzFKJPPUszEhV9Q2Gaqx3bugWTGbqwjC
                            hex!["b21f5aeec1fb8f06f6696fc33505a8c006849f74ed9d29c7c3ebe0dea0fd163e"].unchecked_into(),
                        ), (
                            // 5HQixUgqjLbD6AeB9Lfqa5V3ye3QjcSNam8KoWsrrqFf9FgH
                            hex!["ec71c7615aa56276c9462d682695c99020556f100426d840745f91bc875dff35"].unchecked_into(),
                            // 5DqVgX24AquCfUbL2SWXYSJH5PixAL8ngAmoGkQeVKMMUwLZ
                            hex!["4e654abb844464c97b8b6dd68b7acdec2c3964c6f6baa87a6e1161ebbd35c423"].unchecked_into(),
                        )],
                        hex!["20cd1afa4f95b59b7f61a97360e8bc74a26a6fc13712e6f2eef3a1e020bbcd68"].into(), // 5CoiKRg4hQopwaHxvjdk7C2Gq1pbdvJhsLiYrVHAcsHNkV8m
                        vec![
                            // // 5Ca8HLb1EkJgsDMSpifh33rprTCn9m1DpRkjXNTe7czU1UA5
                            // hex!["167056fa07ae7bc1d36da5520dd8c4c06cb5a5db557986f0c6ae2af0030d4c44"].into(),
                            // // 5G9xD8Bn32KALTZWafYP97e9TwnnzqacwUgSNdLB8mvbvRt1
                            // hex!["b4f179ee3e5e2498eb6d59d1d899bcb0157a917f9cfd8523101e4f78c8a52050"].into()
                        ],
                    )
                },
                vec![
                    MultiaddrWithPeerId::from_str("/ip4/47.244.206.150/tcp/30333/p2p/QmNh3WxHeGATKcEjRSv97HTT6UpmjFdzYSFpn2UeR2DFok").unwrap()
                ],
                Some(
                    TelemetryEndpoints::new(
                        vec![
                            ("https://stats.riochain.io".to_string(), 0)
                        ]
                    ).unwrap()
                ),
                spec.protocol_id,
                spec.properties.clone(),
                None,
            ),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "local" => Some(Alternative::LocalTestnet),
            "test" => Some(Alternative::Testnet),
            "beta" => Some(Alternative::Beta),
            _ => None,
        }
    }
}

fn asset_init(
    init_balance: Vec<(AccountId, Balance)>,
) -> Vec<(
    AssetId,
    rio_assets::AssetAttr,
    Vec<rio_assets::Restriction>,
    Vec<(AccountId, Balance)>,
)> {
    vec![
        // asset id defined in protocol
        (
            AssetId::from(runtime::RFUEL),
            rio_assets::AssetAttr {
                symbol: b"RFUEL".to_vec(),
                name: b"RIO token".to_vec(),
                precision: 8,
                desc: b"RIO chain token".to_vec(),
            },
            vec![],
            init_balance,
        ),
        (
            AssetId::from(runtime::SBTC),
            rio_assets::AssetAttr {
                symbol: b"SBTC".to_vec(),
                name: b"SBTC token".to_vec(),
                precision: 8,
                desc: b"Bitcoin".to_vec(),
            },
            vec![],
            vec![],
        ),
        (
            AssetId::from(runtime::SUSDT),
            rio_assets::AssetAttr {
                symbol: b"S-USDT".to_vec(),
                name: b"USDT token".to_vec(),
                precision: 2,
                desc: b"USDT".to_vec(),
            },
            vec![],
            vec![],
        ),
        (
            AssetId::from(101 as u32),
            rio_assets::AssetAttr {
                symbol: b"RBTC".to_vec(),
                name: b"RBTC token".to_vec(),
                precision: 8,
                desc: b"RBTC".to_vec(),
            },
            vec![],
            vec![],
        ),
        (
            AssetId::from(102 as u32),
            rio_assets::AssetAttr {
                symbol: b"RSC1".to_vec(),
                name: b"RSC1 token".to_vec(),
                precision: 8,
                desc: b"RSC1".to_vec(),
            },
            vec![],
            vec![],
        ),
        (
            AssetId::from(103 as u32),
            rio_assets::AssetAttr {
                symbol: b"RSC2".to_vec(),
                name: b"RSC2 token".to_vec(),
                precision: 8,
                desc: b"RSC2".to_vec(),
            },
            vec![],
            vec![],
        ),
        (
            AssetId::from(104 as u32),
            rio_assets::AssetAttr {
                symbol: b"RSC3".to_vec(),
                name: b"RSC3 token".to_vec(),
                precision: 8,
                desc: b"RSC3".to_vec(),
            },
            vec![],
            vec![],
        ),
        (
            AssetId::from(105 as u32),
            rio_assets::AssetAttr {
                symbol: b"RSC4".to_vec(),
                name: b"RSC4 token".to_vec(),
                precision: 8,
                desc: b"RSC4".to_vec(),
            },
            vec![],
            vec![],
        ),
        (
            AssetId::from(106 as u32),
            rio_assets::AssetAttr {
                symbol: b"RSC5".to_vec(),
                name: b"RSC5 token".to_vec(),
                precision: 8,
                desc: b"RSC5".to_vec(),
            },
            vec![],
            vec![],
        ),
    ]
}

fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    init_balance: Vec<(AccountId, Balance)>,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.to_raw_vec().as_slice().try_into().unwrap(),
                        x.0.to_raw_vec().as_slice().try_into().unwrap(),
                        session_keys(x.0.clone(), x.1.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        rio_assets: Some(RioAssetsConfig {
            root: root_key.clone(),
            init: asset_init(init_balance),
        }),
        // rio_defi_loan: Some(RioDefiLoanConfig {
        //     current_btc_price: 8000_0000,
        //     collateral_asset_id: 10,
        //     loan_asset_id: 1000,
        //     global_ltv_limit: 6500,
        //     global_liquidation_threshold: 9000,
        //     global_warning_threshold: 8000,
        //     next_loan_id: 1,
        //     next_loan_package_id: 1,
        //     pawn_shop: get_account_id_from_seed::<sr25519::Public>("999999999999"),
        //     profit_pool: get_account_id_from_seed::<sr25519::Public>("88888888"),
        //     penalty_rate: 200,
        //     liquidation_account: get_account_id_from_seed::<sr25519::Public>("Bob"),
        //     minimum_collateral: 2000_0000,
        //     liquidation_penalty: 1300,
        // }),
        // rio_defi_saving: Some(RioDefiSavingConfig {
        //     current_phase_id: 1,
        //     collection_asset_id: 10,
        //     share_asset_id: 2,
        //     phase_infos: vec![
        //         (100_00000000, 10000, 3),
        //         (400_00000000, 8000, 4),
        //         (1000_00000000, 5000, 5),
        //         (5000_00000000, 2000, 6),
        //         (10000_00000000, 1000, 7),
        //     ],
        //     collection_account_id: get_account_id_from_seed::<sr25519::Public>("Alice"),
        //     team_account_id: get_account_id_from_seed::<sr25519::Public>("Team"),
        //     profit_pool: get_account_id_from_seed::<sr25519::Public>("88888888"),
        //     profit_asset_id: 8,
        //     reserved_mint_wallet: get_account_id_from_seed::<sr25519::Public>("reserved wallet"),
        //     reserved_mint_asset_id: 8,
        // }),
        rio_gateway: Some(RioGatewayConfig {
            asset_id: AssetId::from(runtime::SBTC),
            threshold: 30_0000_0000,
            admins: vec![(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                runtime::rio_gateway::Auth::All,
            )],
            pending_withdraw_vault: get_account_id_from_seed::<sr25519::Public>("withdraw vault"),
        }),
        rio_payment_fee: Some(RioPaymentFeeConfig {
            account_id: get_account_id_from_seed::<sr25519::Public>("Eve"),
        }),
        // bit_pool: Some(BitPoolConfig {
        //     test_var: 0,
        //     x_percent: 0,
        //     y_percent: 0,
        //     fee_percent: 29,
        //     now_btc_price: 0,
        //     bet_collection_account_id: get_account_id_from_seed::<sr25519::Public>("Alice"),
        //     rake_collection_account_id: get_account_id_from_seed::<sr25519::Public>("Alice"),
        //     admins: vec![(
        //         get_account_id_from_seed::<sr25519::Public>("Alice"),
        //         true,
        //     )],
        // }),
        rio_dapp_dcl: Some(RioDappDclConfig {
            admins: vec![],
            system_account: get_account_id_from_seed::<sr25519::Public>("Alice"),
        }),
        rio_gateway_eth: Some(RioGatewayEthConfig {
            eth_asset_id: 10,
            threshold: 30_0000_0000,
            erc20_token_threshold: vec![(10, 30_0000_0000)],
            admins: vec![(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                runtime::rio_gateway_eth::ETHAuth::All,
            )],
            pending_withdraw_vault: get_account_id_from_seed::<sr25519::Public>("withdraw vault"),
            erc20_token_pending_withdraw_vault: vec![(
                10,
                get_account_id_from_seed::<sr25519::Public>("Alice"),
            )],
        }),
    }
}

fn beta_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.to_raw_vec().as_slice().try_into().unwrap(),
                        x.0.to_raw_vec().as_slice().try_into().unwrap(),
                        session_keys(x.0.clone(), x.1.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        rio_assets: Some(RioAssetsConfig {
            root: root_key.clone(),
            init: asset_init(vec![]),
        }),
        // rio_defi_loan: Some(RioDefiLoanConfig {
        //     current_btc_price: 8000_0000,
        //     collateral_asset_id: 1,
        //     loan_asset_id: 8,
        //     global_ltv_limit: 6500,
        //     global_liquidation_threshold: 9000,
        //     global_warning_threshold: 8000,
        //     next_loan_id: 1,
        //     next_loan_package_id: 1,
        //     pawn_shop: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     profit_pool: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     penalty_rate: 200,
        //     liquidation_account: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     minimum_collateral: 2000_0000,
        //     liquidation_penalty: 1300,
        // }),
        // rio_defi_saving: Some(RioDefiSavingConfig {
        //     current_phase_id: 1,
        //     collection_asset_id: 1,
        //     share_asset_id: 2,
        //     phase_infos: vec![
        //         (100_00000000, 10000, 3),
        //         (400_00000000, 8000, 4),
        //         (1000_00000000, 5000, 5),
        //         (5000_00000000, 2000, 6),
        //         (10000_00000000, 1000, 7),
        //     ],
        //     collection_account_id: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     team_account_id: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     profit_pool: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     profit_asset_id: 8,
        //     reserved_mint_wallet: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     reserved_mint_asset_id: 8,
        // }),
        rio_gateway: Some(RioGatewayConfig {
            asset_id: AssetId::from(runtime::SBTC),
            threshold: 30_0000_0000,
            admins: vec![(
                // 5FJ4MeWhQtvBZxjFkgFm9ojbw2jhqjmjoYo9Jr3xSU7j2Uyk
                hex!["8ee3e606982e20e495b622d091a99d4cdbc669afd8c08efb1acfb375b2e9f61a"].into(),
                runtime::rio_gateway::Auth::All,
            )],
            pending_withdraw_vault: hex![
                "5e1379e6dbd99ec5914c127f7eada6a6a6c773ed47cadf4f023577b0deb3ab09"
            ]
            .into(), // 5EC49JXFCGud9PyhhKxviMdRNHXyn6ZYaAJjsVkLFXYG1WPz
        }),
        rio_payment_fee: Some(RioPaymentFeeConfig {
            account_id: hex!["16bb3aef8cfefcf218ffa6d8722baa4ae808acb0de521e71ccbdeb3bf9cb2d1e"]
                .into(), //5CaWXft7prB292TAjbbFcTbQdrGR8KmwAD8iGPUMHHJPFVdy
        }),
        rio_dapp_dcl: Some(RioDappDclConfig {
            admins: vec![],
            system_account: get_account_id_from_seed::<sr25519::Public>(
                "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
            ),
        }),
        rio_gateway_eth: Some(RioGatewayEthConfig {
            eth_asset_id: 10,
            threshold: 30_0000_0000,
            erc20_token_threshold: vec![(10, 30_0000_0000)],
            admins: vec![(
                get_account_id_from_seed::<sr25519::Public>(
                    "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
                ),
                runtime::rio_gateway_eth::ETHAuth::All,
            )],
            pending_withdraw_vault: get_account_id_from_seed::<sr25519::Public>(
                "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
            ),
            erc20_token_pending_withdraw_vault: vec![(
                10,
                get_account_id_from_seed::<sr25519::Public>(
                    "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
                ),
            )],
        }),
    }
}
