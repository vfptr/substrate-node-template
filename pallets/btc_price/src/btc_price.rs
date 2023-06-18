use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use serde::{Deserialize, };
use sp_core::{offchain::Duration, };
use sp_runtime::{offchain::http,};

#[derive(Deserialize, Encode, Decode, Debug)]
pub struct Price {
	pub usd: u32,
}

#[derive(Deserialize, Encode, Decode, Debug)]
pub struct BtcPrice {
	pub bitcoin: Price,
}

pub fn query_btc_price() -> Result<BtcPrice, http::Error> {
	// prepare for send request
	let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
	let request = http::Request::get(
		"https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd",
	);
	let pending = request
		.add_header("User-Agent", "Substrate-Offchain-Worker")
		.deadline(deadline)
		.send()
		.map_err(|_| http::Error::IoError)?;
	let response = pending.try_wait(deadline).map_err(|_| {
        http::Error::DeadlineReached
    })??;
	if response.code != 200 {
		log::warn!("Unexpected status code: {}", response.code);
		return Err(http::Error::Unknown)
	}
	let body = response.body().collect::<Vec<u8>>();
	let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
		log::warn!("No UTF8 body");
		http::Error::Unknown
	})?;
	// let body_str = r#"{"bitcoin":{"usd":26488}}"#;

	// parse the response str
	let price_info: BtcPrice = serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

	Ok(price_info)
}
