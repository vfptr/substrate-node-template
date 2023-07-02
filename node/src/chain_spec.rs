use hex_literal::hex;
use node_template_runtime::{
	AccountId, BabeConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY, BABE_GENESIS_EPOCH_CONFIG, SessionConfig, StakingConfig, SessionKeys,
	constants::currency::*, StakerStatus, MaxNominations, ImOnlineConfig,
};
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::Perbill;
use node_primitives::*;
use rand::prelude::*;
use sc_telemetry::TelemetryEndpoints;
// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}


/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, BabeId, GrandpaId, ImOnlineId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<BabeId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<ImOnlineId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
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
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online }
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	mut endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
		// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		im_online: ImOnlineConfig { keys: vec![] },

	}
}

fn experimental_network_config_gensis() -> GenesisConfig{
	let wasm_binary = WASM_BINARY.expect(
		"Development wasm binary is not available. This means the client is built with \
		 `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
		 the flag disabled.",
	);
	
	// for i in 1 2 3 4; do for j in stash controller; do subkey inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in babe; do subkey inspect --scheme sr25519 "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in grandpa; do subkey inspect --scheme ed25519 "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in im_online; do subkey inspect --scheme sr25519 "$SECRET//$i//$j"; done; done

	// To insert secret seed:
	// for i in 1 2 3 4; do for j in grandpa; do subkey inspect --scheme ed25519 "$SECRET//$i//$j"; done; done|grep "Secret seed"|awk '{print $3}'|xargs -I {} \
	// sh -c './target/release/node-template key insert --base-path ./tmp/bootnode --chain babe-experiment-raw.json \
	// --scheme ed25519 --key-type grandpa --suri {} ; echo "insert key = {}"'
	let initial_authorities : Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)> = vec![
		(
			hex!["3c61ca34996fbacded522d074733b55be01884ca5d14fcde91240a25feb2561c"].into(),
			hex!["e424db846a19d66812f67e4cd4d2562ec284d15710117a2c03d13fc375f5575d"].into(),
			hex!["74f177daebd58b5cc22089a265ccbf1b06b85bc060da79e3a6720924d7a87c7f"].unchecked_into(),
			hex!["3738acbad98e943a28e73b89560814880173d40a3ac6a1fa25554afa67a2d079"].unchecked_into(),
			hex!["fe2a4c73c7f18abdea3af143e568e34fd3b97d6fa6bbd2bd145e2b82e699fb7a"].unchecked_into(),
		),
		(
			hex!["b83b04cbd09542994294e3b9d2169c427875ba90cf4597314dca4c4fa7dd353e"].into(),
			hex!["88a82980e52d2489777b8dc3f7ea155682868e5cd679b8377098ced0ae82c734"].into(),
			hex!["587822b277e51453a362d513628a3d825ae697cf8f8d47a4cd1de74834996b11"].unchecked_into(),
			hex!["157d5ef2694141f7fdecad1948bf9b70950c59abd923c8b502af531f7e807b39"].unchecked_into(),
			hex!["d24918ee818bf3bbfa07f37b0c0567dc54186da8ea3be879e8bcebb826421a33"].unchecked_into(),
		),
		(
			hex!["1294dc35fe1b3741cd9a44aaf67656a14d8ae22aed6613de353cc8e2718bdb4c"].into(),
			hex!["c855f517ae9859665abb32e26be9ca39982ba12fa678b6250398ee7e3dba2f19"].into(),
			hex!["74c088e11e3c23caebdbe4b3882e30e2503d470982b3b1b9a1d3e30eaec6d951"].unchecked_into(),
			hex!["f161624d59e749f626b28042e598c281be20acc0111bd71c96e3b2f4dbed64e4"].unchecked_into(),
			hex!["44024801d332ac24e22a49eaada7b8a0724942c99dafe2033107bf2ecf80502a"].unchecked_into(),
		),
		(
			hex!["1e7ee3c90b6c7b41f3712b018f099e5ea2c5d392895a7c9f04c0e6f48cc1344e"].into(),
			hex!["f81239e7e6c8aa3c6aaef85fc1d79a405915648d780154d15ac7229949d7d825"].into(),
			hex!["ec7da4cd47ff6c75daeb18248a31dc5b716e2674d9c0a0d12488cfd74c31ab63"].unchecked_into(),
			hex!["bda349cd8769fc0bf4e054707cb0807a70bd1c2992d8fc09916a17adddeeddb6"].unchecked_into(),
			hex!["8af53447d05e3f9e1f8784025aa4ede96fb30a48a83a5589dab3a8013e98fe74"].unchecked_into(),
		),
	];
	// generated with secret: subkey inspect "$secret"/fir
	let root_key: AccountId = hex!["d87938ff5402a9fee67c248e1fbd039fea7058b587a65deda2af79e6274e4163"].into();
	let endowed_accounts: Vec<AccountId> = vec![];
	testnet_genesis(wasm_binary, initial_authorities, vec![], root_key, endowed_accounts, true)
}


const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
pub fn experimental_network_config() -> ChainSpec {
	let boot_nodes = vec![];

	ChainSpec::from_genesis(
		"Substrate Babe experiment",
		"babe-experiment-by-Joe",
		ChainType::Live,
		experimental_network_config_gensis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		None,
		Default::default(),
	)
}