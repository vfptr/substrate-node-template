#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// Define the pallet module using the frame_support::pallet macro
// A pallet for proof of existence.
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{OptionQuery, *},
		traits::Randomness,
		Blake2_128Concat,
	};
	use sp_io::hashing::blake2_128;

	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	// Define the pallet struct using the pallet::pallet macro
	#[pallet::pallet]
	pub struct Pallet<T>(_);


	// Define the kitty struct.
	pub type KittyId = u32;
	#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);


	// Define the kitty id for storing, increasing 1 every time a kitty is created or breed.
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	// Define the storage for storing kitty.
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	// Define storage for storing a Kitty's parents ID
	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	// Define sotrage for stroing a Kitty's owner.
	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	// Define the pallet's configuration trait
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Define the events that can be emitted by the pallet
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBreed { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferd { who: T::AccountId, dest: T::AccountId, kitty_id: KittyId},
	}

	// Define the errors that can be returned by the pallet
	#[pallet::error]
	pub enum Error<T> {
		NotKittyOwner,
		InvalidKittyId,
		SameKittyId,
	}

	// Define the pallet's dispatchable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Create a Kitty from the current id storaged on Chain, 
		// the id is increased every time this menthod is called.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_kittyid_and_update_next()?;
			let kitty = Kitty(Self::random_value(&who));
			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			Self::deposit_event(Event::<T>::KittyCreated { who, kitty_id, kitty });
			Ok(())
		}

		// Breed a Kitty from the current id storaged on Chain, whose id is affacted by those of two parents.
		// the current kitty id is increased every time this menthod is called.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			let kitty_id = Self::get_kittyid_and_update_next()?;
			let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let mut data = [0u8; 16];
			let new_id = Self::random_value(&who);
			for i in 0..kitty_1.0.len() {
				data[i] = (kitty_1.0[i] & new_id[i]) | (kitty_2.0[i] & !new_id[i]);
			}
			let kitty = Kitty(data);
			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));
			Self::deposit_event(Event::<T>::KittyBreed { who, kitty_id, kitty });
			Ok(())
		}

		// Transfer a Kitty's onwership to another one.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer(origin: OriginFor<T>, dest: T::AccountId, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(KittyOwner::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner == who, Error::<T>::NotKittyOwner);
			
			KittyOwner::<T>::insert(kitty_id, &dest);
			Self::deposit_event(Event::<T>::KittyTransferd { who, dest, kitty_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// The NextKittyId is stored, which is increased every time this method is called.
		// It starts from 0.
		fn get_kittyid_and_update_next() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id
					.checked_add(1)
					.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}

		// Generate a random value using Randomness strait.
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				frame_system::Pallet::<T>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}
	}
}
