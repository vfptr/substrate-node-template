use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_create() {
	new_test_ext().execute_with(|| {
		let kitty_id = Kitty::next_kitty_id();
		let account_id = 10000;
		assert_ok!(Kitty::create(RuntimeOrigin::signed(account_id)));
		let created_kitty = Kitty::kitties(kitty_id);
		assert!(created_kitty.is_some());
		assert_eq!(Kitty::kitty_owner(kitty_id), Some(account_id));
		assert_eq!(Kitty::kitty_parents(kitty_id), None);
		let e = Event::<Test>::KittyCreated{
			who: account_id, 
			kitty_id, 
			kitty: created_kitty.unwrap()}.into();
		System::assert_last_event(e);
	});
}

#[test]
fn create_failed_when_kitty_id_too_large() {
	new_test_ext().execute_with(|| {
		let account_id = 10000;

		crate::NextKittyId::<Test>::set(crate::KittyId::max_value());
		assert_noop!(
			Kitty::create(RuntimeOrigin::signed(account_id)),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn kitty_id_increased_after_create() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 10000;
		assert_eq!(Kitty::next_kitty_id(), kitty_id);
		assert_ok!(Kitty::create(RuntimeOrigin::signed(account_id)));
		assert_eq!(Kitty::next_kitty_id(), kitty_id + 1);
	});
}

#[test]
fn breed_kitty_failed_when_parents_id_same() {
	new_test_ext().execute_with(|| {
		let kitty_id_1 = 0;
		let kitty_id_2 = 0;
		let account_id = 10000;
		assert_noop!(
			Kitty::breed(RuntimeOrigin::signed(account_id), kitty_id_1, kitty_id_2),
			Error::<Test>::SameKittyId
		);
	});
}

#[test]
fn breed_kitty_failed_when_parents_not_exists() {
	new_test_ext().execute_with(|| {
		let account_id = 10000;
		let one_parent_id = Kitty::next_kitty_id();
		let _ = Kitty::create(RuntimeOrigin::signed(account_id));
		assert!(Kitty::kitties(one_parent_id).is_some());
		assert_noop!(
			Kitty::breed(RuntimeOrigin::signed(account_id), one_parent_id, one_parent_id + 1),
			Error::<Test>::InvalidKittyId
		);
		assert_noop!(
			Kitty::breed(RuntimeOrigin::signed(account_id), one_parent_id + 1, one_parent_id),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn breed_kitty_works() {
	new_test_ext().execute_with(|| {
		let kitty_id_1 = Kitty::next_kitty_id();
		let account_id = 10000;
		let _ = Kitty::create(RuntimeOrigin::signed(account_id));
		let kitty_id_2 = Kitty::next_kitty_id();
		let _ = Kitty::create(RuntimeOrigin::signed(account_id));
		let kitty_child_id = Kitty::next_kitty_id();
		assert_ok!(Kitty::breed(RuntimeOrigin::signed(account_id), kitty_id_1, kitty_id_2));
		let kitty_child = Kitty::kitties(kitty_child_id);
		assert!(kitty_child.is_some());
		assert_eq!(Kitty::next_kitty_id(), kitty_child_id + 1);
		assert_eq!(Kitty::kitty_owner(kitty_child_id), Some(account_id));
		assert_eq!(Kitty::kitty_parents(kitty_child_id).unwrap(), (kitty_id_1, kitty_id_2));
		let e = Event::<Test>::KittyBreed{
			who: account_id, 
			kitty_id : kitty_child_id, 
			kitty: kitty_child.unwrap()}.into();
		System::assert_last_event(e);
	});
}

#[test]
fn transfer_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		let kitty_id_1 = Kitty::next_kitty_id();
		let account_id = 10000;
		let account_id_another = 10001;
		let account_dest = 10002;
		let _ = Kitty::create(RuntimeOrigin::signed(account_id));
		assert_noop!(
			Kitty::transfer(RuntimeOrigin::signed(account_id_another), account_dest, kitty_id_1),
			Error::<Test>::NotKittyOwner
		);
	});
}

#[test]
fn transfer_failed_invalid_kitty_id() {
	new_test_ext().execute_with(|| {
		let kitty_id_1 = Kitty::next_kitty_id();
		let account_id = 10000;
		let account_dest = 10002;
		let _ = Kitty::create(RuntimeOrigin::signed(account_id));
		assert_noop!(
			Kitty::transfer(RuntimeOrigin::signed(account_id), account_dest, kitty_id_1 + 1),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn transfer_kitty_works() {
	new_test_ext().execute_with(|| {
		let kitty_id_1 = Kitty::next_kitty_id();
		let account_id = 10000;
		let account_dest = 10002;
		let _ = Kitty::create(RuntimeOrigin::signed(account_id));
		assert_ok!(
			Kitty::transfer(RuntimeOrigin::signed(account_id), account_dest, kitty_id_1),
		);
		assert_eq!(Kitty::kitty_owner(kitty_id_1), Some(account_dest));
		let e = Event::<Test>::KittyTransferd {
			who: account_id, 
			dest: account_dest, 
			kitty_id: kitty_id_1};
		System::assert_last_event(e.into());
	});
}
