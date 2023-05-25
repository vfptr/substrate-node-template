#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// Define the pallet module using the frame_support::pallet macro
// A pallet for proof of existence.
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;

    // Define the pallet struct using the pallet::pallet macro
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Define the pallet's configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        type MaxClaimLength: Get<u32>;
    }

    // Define the storage item for storing the claims
    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type Proofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxClaimLength>,
        (T::AccountId, T::BlockNumber)
    >;

    // Define the events that can be emitted by the pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
        ClaimTransfered(T::AccountId, T::AccountId, Vec<u8>)
    }

    // Define the errors that can be returned by the pallet
    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExists,
        ClaimTooLong,
        ClaimNotExists,
        NotClaimOwner,
    }

    // Define the pallet's dispatchable functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Create a new claim
        #[pallet::weight(0)]
        pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
            // Verify that the transaction is signed by a valid account
            let who = ensure_signed(origin)?;
            // Create a bounded vector from the claim data
            let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(
                |_| Error::<T>::ClaimTooLong
            )?;
            // Ensure that the claim does not already exist
            ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExists);
            // Store the claim along with the owner's account ID and the current block number
            Proofs::<T>::insert(
                &bounded_claim,
                (who.clone(), frame_system::Pallet::<T>::block_number())
            );
            // Emit the ClaimCreated event
            Self::deposit_event(Event::ClaimCreated(who, claim));
            Ok(())
        }

        // Revoke an existing claim
        #[pallet::weight(0)]
        pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
            // Verify that the transaction is signed by a valid account
            let sender = ensure_signed(origin)?;

			// Ensure that the claim is not too long.
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(
				|_| Error::<T>::ClaimTooLong
			)?;

			// Get the owner of the claim and ensure that it exists.
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExists)?;

			// Ensure that the sender is the owner of the claim.
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// Remove the claim.
			Proofs::<T>::remove(&bounded_claim);

			// Emit a ClaimRevoked event.
			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(()) 
		}
		
		/// transfer the claim from a account id to another account id.
		#[pallet::weight(0)]
		pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResult{
			// Ensure that the transaction is signed by the sender.
			let sender = ensure_signed(origin)?;

			// Ensure that the claim is not too long.
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(
				|_| Error::<T>::ClaimTooLong
			)?;

			// Get the owner of the claim and ensure that it exists.
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExists)?;

			// Ensure that the sender is the owner of the claim.
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// Transfer the claim to the new owner.
			Proofs::<T>::insert(
				&bounded_claim,
				(dest.clone(), frame_system::Pallet::<T>::block_number())
			);

			// Emit a ClaimTransfered event.
			Self::deposit_event(Event::ClaimTransfered(sender, dest, claim));

			Ok(().into())
		}
	}
}