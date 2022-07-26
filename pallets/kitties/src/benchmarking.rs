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
	// do brenchmark with mock runtime
	impl_benchmark_test_suite!(Kitty, crate::mock::new_test_ext(), crate::mock::Test);
}
