//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Kitty;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	bm_create_kity {
		let dna: Vec<u8> = b"Hung Pham".to_vec();
		let caller: T::AccountId = whitelisted_caller();
	}: create_new_kitty(RawOrigin::Signed(caller), dna)
	// Check
	verify {
		assert_eq!(KittyId::<T>::get(), 1);
	}

	bm_transfer_kity {
		let dna: Vec<u8> = b"Hung Pham".to_vec();
		let from_user: T::AccountId = whitelisted_caller();
		let from_user_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(from_user.clone()));
		Kitty::<T>::create_new_kitty(from_user_origin, dna.clone());
		// let to_user: T::AccountId = account("receiver", 0 , 0);
		let to_user: T::AccountId = whitelisted_caller();
	}: transfer_kitty_to_friend(RawOrigin::Signed(from_user.clone()), to_user.clone(), dna.clone())
	// Check
	verify {
		assert_eq!(KittyId::<T>::get(), 1);
		assert_eq!(KittiesOwned::<T>::get(from_user).len(),0);
		assert_eq!(KittiesOwned::<T>::get(to_user).len(),1);
	}

	// do brenchmark with mock runtime
	impl_benchmark_test_suite!(Kitty, crate::mock::new_test_ext(), crate::mock::Test);
}
