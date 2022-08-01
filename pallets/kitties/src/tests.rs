use crate::{mock::*, Error, Event as KittyEvent};
use frame_support::{assert_noop, assert_ok};
use frame_system::{ Origin, RawOrigin, ensure_signed};

fn last_event() -> KittyEvent<Test> {
	println!("Events:{:?}", System::events());
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let Event::KittyModule(inner) = e { Some(inner) } else { None })
		.last()
		.unwrap()
}

#[test]
fn it_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let from_user = RawOrigin::Signed(1u64);
		let from_user_origin = from_user.into();
		assert_ok!(KittyModule::create_new_kitty(from_user_origin, b"hello".to_vec()));
	});
}

#[test]
fn transfer_kitty_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let from_account_id = 1u64;
		let from_user = RawOrigin::Signed(from_account_id);
		let from_user_origin = from_user.into();

		let to_account_id = 2u64;

		assert_ok!(KittyModule::create_new_kitty(from_user_origin, b"hello".to_vec()));
		let kitties_owned_by_from_user = KittyModule::kitty_owned(from_account_id);
		assert_eq!(kitties_owned_by_from_user.len(),1);
		let dns_hash = kitties_owned_by_from_user[0];
		assert_ok!(KittyModule::transfer_kitty_to_friend(RawOrigin::Signed(from_account_id).into(), to_account_id, dns_hash));

		let kitties_owned_by_from_user = KittyModule::kitty_owned(from_account_id);
		assert_eq!(kitties_owned_by_from_user.len(),0);
		let kitties_owned_by_to_user = KittyModule::kitty_owned(to_account_id);
		assert_eq!(kitties_owned_by_to_user.len(),1);

		assert_eq!(KittyModule::kitty_id(),1);
	});

}

#[test]
fn transfer_kitty_should_fail_cause_of_wrong_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let from_account_id = 1u64;
		let from_user = RawOrigin::Signed(from_account_id);
		let from_user_origin = from_user.into();

		let to_account_id = 2u64;

		assert_ok!(KittyModule::create_new_kitty(from_user_origin, b"hello".to_vec()));
		let kitties_owned_by_from_user = KittyModule::kitty_owned(from_account_id);
		assert_eq!(kitties_owned_by_from_user.len(),1);
		let dns_hash = kitties_owned_by_from_user[0];
		assert_noop!(KittyModule::transfer_kitty_to_friend(RawOrigin::Signed(to_account_id).into(), to_account_id, dns_hash),
		Error::<Test>::KittyWrongOwner
		);
	});

}

#[test]
fn transfer_kitty_should_fail_cause_of_send_myself() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let from_account_id = 1u64;
		let from_user = RawOrigin::Signed(from_account_id);
		let from_user_origin = from_user.into();

		let to_account_id = from_account_id;

		assert_ok!(KittyModule::create_new_kitty(from_user_origin, b"hello".to_vec()));
		let kitties_owned_by_from_user = KittyModule::kitty_owned(from_account_id);
		assert_eq!(kitties_owned_by_from_user.len(),1);
		let dns_hash = kitties_owned_by_from_user[0];
		assert_noop!(KittyModule::transfer_kitty_to_friend(RawOrigin::Signed(from_account_id).into(), to_account_id, dns_hash),
		Error::<Test>::KittyTransferFail
		);
	});

}
