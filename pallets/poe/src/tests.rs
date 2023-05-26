use crate::{mock::*, Error, Proofs};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let bounded_claim = BoundedVec::try_from(claim.clone()).unwrap();

		assert_ok!(Poe::create_claim(RuntimeOrigin::signed(1), claim));
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn create_claim_failed_when_claim_too_long() {
	new_test_ext().execute_with(|| {
		let claim = Vec::from([1; 513]);

		assert_noop!(Poe::create_claim(RuntimeOrigin::signed(1), claim), Error::<Test>::ClaimTooLong);
	});
}


#[test]
fn create_claim_failed_when_claim_already_exists() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2];
		let _ = Poe::create_claim(RuntimeOrigin::signed(1), claim.clone());

		assert_noop!(
			Poe::create_claim(RuntimeOrigin::signed(1), claim),
			Error::<Test>::ProofAlreadyExists
		);
	})
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2];
		let _ = Poe::create_claim(RuntimeOrigin::signed(1), claim.clone());

		assert_ok!(Poe::revoke_claim(RuntimeOrigin::signed(1), claim));
	});
}

#[test]
fn revoke_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2];

		assert_noop!(
			Poe::revoke_claim(RuntimeOrigin::signed(1), claim),
			Error::<Test>::ClaimNotExists
		);
	});
}

#[test]
fn revoke_claim_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2];
		let _ = Poe::create_claim(RuntimeOrigin::signed(1), claim.clone());

		assert_noop!(
			Poe::revoke_claim(RuntimeOrigin::signed(2), claim),
			Error::<Test>::NotClaimOwner
		);
	});
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2];
		let _ = Poe::create_claim(RuntimeOrigin::signed(1), claim.clone());
		let bounded_claim = BoundedVec::try_from(claim.clone()).unwrap();

		assert_ok!(Poe::transfer_claim(RuntimeOrigin::signed(1), claim, 2));
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn transfer_claim_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2];
		let _ = Poe::create_claim(RuntimeOrigin::signed(1), claim.clone());

		assert_noop!(
			Poe::transfer_claim(RuntimeOrigin::signed(2), claim, 3),
			Error::<Test>::NotClaimOwner
		);
	});
}

#[test]
fn transfer_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![1, 2];

		assert_noop!(
			Poe::transfer_claim(RuntimeOrigin::signed(1), claim, 2),
			Error::<Test>::ClaimNotExists
		);
	});
}