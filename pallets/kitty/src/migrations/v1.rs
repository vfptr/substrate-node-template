use frame_support::{
	pallet_prelude::*, storage::StoragePrefixedMap, traits::GetStorageVersion, weights::Weight,
};

use crate::pallet::*;
use frame_support::{migration::storage_key_iter, Blake2_128Concat};

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct KittyOld(pub [u8; 16]);

// migrate Kitty structure from v0 to v1.
pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();
	if on_chain_version != 0 || current_version != 1 {
		return Weight::zero()
	}

	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();
	for (i, kitty) in storage_key_iter::<KittyId, KittyOld, Blake2_128Concat>(module, item).drain()
	{
		let kitty = Kitty { dna: kitty.0, name: *b"0->1" };
		Kitties::<T>::insert(i, &kitty);
	}
	current_version.put::<Pallet::<T>>();
	return Weight::zero()
}
