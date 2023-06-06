use crate::{mock::*, Config, Error, Event};
use frame_support::{assert_noop, assert_ok, traits::Currency};

mod create_fun {
	use super::*;
	#[test]
	fn it_works_for_create() {
		new_test_ext().execute_with(|| {
			let kitty_id = Kitty::next_kitty_id();
			let account_id = 10000;
			let who = RuntimeOrigin::signed(account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);
			let balance = <Test as Config>::Currency::free_balance(&account_id);
			assert_eq!(init_balance, balance);
			assert_ok!(Kitty::create(who));
			let balance = <Test as Config>::Currency::free_balance(&account_id);
			assert_eq!(init_balance - price, balance);

			let created_kitty = Kitty::kitties(kitty_id);
			assert!(created_kitty.is_some());
			assert_eq!(Kitty::kitty_owner(kitty_id), Some(account_id));
			assert_eq!(Kitty::kitty_parents(kitty_id), None);
			let e = Event::<Test>::KittyCreated {
				who: account_id,
				kitty_id,
				kitty: created_kitty.unwrap(),
			}
			.into();
			System::assert_last_event(e);
		});
	}

	#[test]
	fn create_fail_when_insufficient_balance() {
		new_test_ext().execute_with(|| {
			let account_id = 10000;
			let who = RuntimeOrigin::signed(account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance() - 1;
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);
			let balance = <Test as Config>::Currency::free_balance(&account_id);
			assert_eq!(init_balance, balance);
			assert_noop!(Kitty::create(who), Error::<Test>::InsufficientBalance);
		})
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
			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);
			assert_eq!(Kitty::next_kitty_id(), kitty_id);
			assert_ok!(Kitty::create(RuntimeOrigin::signed(account_id)));
			assert_eq!(Kitty::next_kitty_id(), kitty_id + 1);
		});
	}
}

mod breed_fun {
	use super::*;
	#[test]
	fn breed_fail_when_insufficient_balance() {
		new_test_ext().execute_with(|| {
			let kitty_id_1 = Kitty::next_kitty_id();
			let account_id = 10000;
			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price * 2 + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);

			let _ = Kitty::create(RuntimeOrigin::signed(account_id));
			let kitty_id_2 = Kitty::next_kitty_id();
			let _ = Kitty::create(RuntimeOrigin::signed(account_id));

			let init_balance = price - 1;
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);
			let balance = <Test as Config>::Currency::free_balance(&account_id);
			assert_eq!(init_balance + <Test as Config>::Currency::minimum_balance(), balance);

			assert_noop!(
				Kitty::breed(RuntimeOrigin::signed(account_id), kitty_id_1, kitty_id_2),
				Error::<Test>::InsufficientBalance
			);
		})
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

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price * 2 + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);

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
			
			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price * 3 + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);

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
			let e = Event::<Test>::KittyBreed {
				who: account_id,
				kitty_id: kitty_child_id,
				kitty: kitty_child.unwrap(),
			}
			.into();
			System::assert_last_event(e);
		});
	}
}

mod transfer_fun {
	use super::*;
	#[test]
	fn transfer_failed_when_not_owner() {
		new_test_ext().execute_with(|| {
			let kitty_id_1 = Kitty::next_kitty_id();
			let account_id = 10000;
			let account_id_another = 10001;
			let account_dest = 10002;

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price * 2 + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);

			let _ = Kitty::create(RuntimeOrigin::signed(account_id));
			assert_noop!(
				Kitty::transfer(
					RuntimeOrigin::signed(account_id_another),
					account_dest,
					kitty_id_1
				),
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

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price * 2 + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&account_id, init_balance);

			let _ = Kitty::create(RuntimeOrigin::signed(account_id));
			assert_ok!(Kitty::transfer(
				RuntimeOrigin::signed(account_id),
				account_dest,
				kitty_id_1
			),);
			assert_eq!(Kitty::kitty_owner(kitty_id_1), Some(account_dest));
			let e = Event::<Test>::KittyTransferd {
				who: account_id,
				dest: account_dest,
				kitty_id: kitty_id_1,
			};
			System::assert_last_event(e.into());
		});
	}
}

mod sale_fun {
	use super::*;
	#[test]
	fn sale_fail_when_not_owner() {
		new_test_ext().execute_with(|| {
			let buyer_account_id = 10000;
			let not_onwer_account_id = 10001;
			let saler = RuntimeOrigin::signed(not_onwer_account_id);
			let not_owner = RuntimeOrigin::signed(buyer_account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ =
				<Test as Config>::Currency::deposit_creating(&not_onwer_account_id, init_balance);
			let kitty_id = Kitty::next_kitty_id();
			assert_ok!(Kitty::create(saler.clone()));
			assert_noop!(Kitty::sale(not_owner, kitty_id), Error::<Test>::NotKittyOwner);
		})
	}

	#[test]
	fn sale_fail_when_already_on_market() {
		new_test_ext().execute_with(|| {
			let saler_account_id = 10000;
			let saler = RuntimeOrigin::signed(saler_account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&saler_account_id, init_balance);
			let kitty_id = Kitty::next_kitty_id();
			assert_ok!(Kitty::create(saler.clone()));
			let _ = Kitty::sale(saler.clone(), kitty_id);
			assert_noop!(Kitty::sale(saler, kitty_id), Error::<Test>::AlreadyOnSale);
		})
	}

	#[test]
	fn sale_works() {
		new_test_ext().execute_with(|| {
			let saler_account_id = 10000;
			let saler = RuntimeOrigin::signed(saler_account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&saler_account_id, init_balance);
			let kitty_id = Kitty::next_kitty_id();
			assert_ok!(Kitty::create(saler.clone()));
			assert_ok!(Kitty::sale(saler, kitty_id));

			let e = Event::<Test>::KittyOnSale { who: saler_account_id, kitty_id };
			System::assert_last_event(e.into())
		})
	}
}

mod buy_fun {
	use super::*;

	#[test]
	fn buy_fail_when_insufficient_balance() {
		new_test_ext().execute_with(|| {
			let buyer_account_id = 10000;
			let saler_account_id = 10001;
			let saler = RuntimeOrigin::signed(saler_account_id);
			let buyer = RuntimeOrigin::signed(buyer_account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&saler_account_id, init_balance);
			let _ =
				<Test as Config>::Currency::deposit_creating(&buyer_account_id, init_balance - 1);
			let kitty_id = Kitty::next_kitty_id();
			assert_ok!(Kitty::create(saler.clone()));
			let _ = Kitty::sale(saler, kitty_id);
			assert_noop!(Kitty::buy(buyer, kitty_id), Error::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn buy_fail_when_not_on_sale(){
		new_test_ext().execute_with(||{
			let buyer_account_id = 10000;
			let saler_account_id = 10001;
			let saler = RuntimeOrigin::signed(saler_account_id);
			let buyer = RuntimeOrigin::signed(buyer_account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&saler_account_id, init_balance);
			let _ =
				<Test as Config>::Currency::deposit_creating(&buyer_account_id, init_balance);
			let kitty_id = Kitty::next_kitty_id();
			assert_ok!(Kitty::create(saler.clone()));
			assert_noop!(Kitty::buy(buyer, kitty_id), Error::<Test>::NotOnSale);
		})
	}

	#[test]
	fn buy_fail_when_already_owned(){
		new_test_ext().execute_with(|| {
			let saler_account_id = 10001;
			let saler = RuntimeOrigin::signed(saler_account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&saler_account_id, init_balance);
			let kitty_id = Kitty::next_kitty_id();
			assert_ok!(Kitty::create(saler.clone()));
			let _ = Kitty::sale(saler.clone(), kitty_id);
			assert_noop!(Kitty::buy(saler, kitty_id), Error::<Test>::AlreadyOwned);
		})
	}

	
	#[test]
	fn it_works_for_buy() {
		new_test_ext().execute_with(|| {
			let buyer_account_id = 10000;
			let saler_account_id = 10001;
			let saler = RuntimeOrigin::signed(saler_account_id);
			let buyer = RuntimeOrigin::signed(buyer_account_id);

			let price = <Test as Config>::KittyPrice::get();
			let init_balance = price + <Test as Config>::Currency::minimum_balance();
			let _ = <Test as Config>::Currency::deposit_creating(&saler_account_id, init_balance);
			let _ =
				<Test as Config>::Currency::deposit_creating(&buyer_account_id, init_balance);
			let kitty_id = Kitty::next_kitty_id();
			assert_ok!(Kitty::create(saler.clone()));
			let _ = Kitty::sale(saler, kitty_id);
			assert_ok!(Kitty::buy(buyer, kitty_id));

			let e = Event::<Test>::KittyBought { who: buyer_account_id, kitty_id: kitty_id };
			System::assert_last_event(e.into());
		})
	}

}
