use crate::*;
use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	use super::*;
	fn confirm_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
		frame_system::Pallet::<T>::assert_last_event(generic_event.into());
	}
	benchmarks! {
		// if the name of this item is as same as the function name to benchmark, use _ instead.
		create_claim {
			let d in 0.. T::MaxClaimLength::get();
			let claim = vec![0; d as usize];
			let caller: T::AccountId = whitelisted_caller();
		}: _(RawOrigin::Signed(caller.clone()), claim.clone())

		// to verify the result is as expected.
		verify {
			confirm_last_event::<T>(Event::ClaimCreated(caller, claim).into())
		}

		revoke_claim {
            let d in 0 .. T::MaxClaimLength::get();
            let claim = vec![0; d as usize];
            let caller: T::AccountId = whitelisted_caller();
            assert!(Pallet::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()).is_ok());
        }: _(RawOrigin::Signed(caller.clone()), claim.clone())

        verify {
            confirm_last_event::<T>(Event::ClaimRevoked(caller, claim).into())
        }

        transfer_claim {
            let d in 0 .. T::MaxClaimLength::get();
            let claim = vec![0; d as usize];
            let caller: T::AccountId = whitelisted_caller();
            let target: T::AccountId = account("target", 0, 0);
            assert!(Pallet::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()).is_ok());
        }: _(RawOrigin::Signed(caller.clone()), claim.clone(), target)

		impl_benchmark_test_suite!(Poe, crate::mock::new_test_ext(), crate::mock::Test);
	}
}
