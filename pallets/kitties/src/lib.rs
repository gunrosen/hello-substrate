#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_support::inherent::Vec;
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
pub type Id = u32;
use frame_support::traits::Currency;
use frame_support::traits::Time;
use frame_support::traits::Get;
use frame_support::traits::Randomness as RandomnessT;
use frame_support::dispatch::fmt;
use sp_runtime::traits::Hash;
use sp_runtime::SaturatedConversion;


type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	// use std;
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T:Config> {
		dna: T::Hash,
		name: Vec<u8>,
		owner: T::AccountId,
		price: BalanceOf<T>,
		gender: Gender,
		created_date: u64,
	}
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
	#[scale_info(skip_type_params(T))]
	pub struct RandomInfo<T:Config> {
		hash: T::Hash,
		block_number: T::BlockNumber,
	}

	impl<T: Config> Default for RandomInfo<T> {
		fn default() -> Self {
			RandomInfo { hash: T::Hash::default(), block_number: T::BlockNumber::default() }
		}
	}

	impl<T:Config> fmt::Debug for Kitty<T>{
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
				.field("name", &self.name)
				.field("dna", &self.dna)
				.field("owner", &self.owner)
				.field("price", &self.price)
				.field("gender", &self.gender)
				.field("created_date", &self.created_date)
				.finish()
		}
	}
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Gender {
		Male,
		Female,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type TimeProvider: Time;
		#[pallet::constant]
		type KittyLimit: Get<u32>;
		type RandomProvider: RandomnessT<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn kitty_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type KittyId<T> = StorageValue<_, Id, ValueQuery>;

	// Map from dna to Kitty
	#[pallet::storage]
	#[pallet::getter(fn get_kitty)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>, OptionQuery>;

	// Store ownership of account Id to kittes
	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub(super) type Nonce<T> = StorageValue<_, Id, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_random_number)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type RandomNumber<T: Config> = StorageValue<_, RandomInfo<T>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyCreated { kitty: Vec<u8>, dna: T::Hash, owner: T::AccountId },
		KittyTransferred { from: T::AccountId, to: T::AccountId, kitty:Vec<u8>, dna: T::Hash },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		KittyDnaAlreadyExist,
		KittyNotFound,
		KittyTransferFail,
		KittyWrongOwner,
		KittyOwnedTooLarge,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_kitty_name: Vec<Vec<u8>>,
		pub alice_account: Option<T::AccountId>,
		pub initial_time: u64,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				initial_kitty_name: Vec::new(),
				alice_account: None,
				initial_time: 0,
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			log::info!("JUMP to pallet_kitties genesis build");
			if let Some(alice) = &self.alice_account {
				for kitty_name in self.initial_kitty_name.iter() {
					let kitty = Kitty::<T> {
						owner: alice.clone(),
						name: kitty_name.clone(),
						dna: <T as frame_system::Config>::Hashing::hash(&kitty_name),
						gender: Pallet::<T>::calculate_gender(kitty_name.to_vec()).unwrap(),
						created_date: self.initial_time,
						price: 99u32.into(),
					};
					let current_id = KittyId::<T>::get();
					let next_id = current_id + 1;
					KittiesOwned::<T>::append(&alice, kitty.dna.clone());
					Kitties::<T>::insert(kitty.dna.clone(), kitty);
					KittyId::<T>::put(next_id);
				}
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(35_802_000 + T::DbWeight::get().reads_writes(4,3))]
		pub fn create_new_kitty(origin: OriginFor<T>, dna: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			log::info!("total balance:{:?}", T::Currency::total_balance(&who));
			let gender = Self::calculate_gender(dna.clone())?;
			// Get nonce_encoded
			let nonce = Nonce::<T>::get();
			Nonce::<T>::put(nonce.wrapping_add(1));
			// let nonce_encoded = nonce.encode();
			// log::info!("nonce:{:?} and nonce_encoded:{:?}", nonce, nonce_encoded);
			// let (dna_random, block_number) = T::RandomProvider::random(&nonce_encoded);
			// log::info!("random at block_number:{:?}", block_number);
			let dna_random = Pallet::<T>::gen_dna(dna.clone());
			let created_time = T::TimeProvider::now().saturated_into::<u64>();
			let kitty = Kitty::<T> { name: dna.clone(), dna: dna_random.clone(), price: 0u32.into(), gender, owner: who.clone(), created_date: created_time  };
            ensure!(!Kitties::<T>::contains_key(&kitty.dna), Error::<T>::KittyDnaAlreadyExist);
			ensure!(KittiesOwned::<T>::get(&who).len() < T::KittyLimit::get() as usize, Error::<T>::KittyOwnedTooLarge);
            let current_id = KittyId::<T>::get();
            let next_id = current_id + 1;

			log::info!("new kitty: {:?}", kitty);
			// Update storage.
			KittiesOwned::<T>::append(&who, kitty.dna.clone());
			Kitties::<T>::insert(kitty.dna.clone(), kitty);
			KittyId::<T>::put(next_id);

			// Emit an event.
			Self::deposit_event(Event::KittyCreated{kitty: dna.clone(), dna: dna_random, owner: who.clone()});
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Put number into storage map
		#[pallet::weight(47_000_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn transfer_kitty_to_friend(origin: OriginFor<T>, to: T::AccountId, dna: T::Hash) -> DispatchResult{
			let from = ensure_signed(origin)?;
			let mut kitty = Kitties::<T>::get(&dna).ok_or(Error::<T>::KittyNotFound)?;
			ensure!(kitty.owner == from, Error::<T>::KittyWrongOwner);
			ensure!(kitty.owner != to, Error::<T>::KittyTransferFail);
			let mut from_owned = KittiesOwned::<T>::get(&from);
			let exist_dna = from_owned.iter().position(|ids| *ids == dna);
			match exist_dna {
				Some(d) => {from_owned.swap_remove(d);},
				None => {panic!("KittyNotFound");}
			}
			let mut to_owned = KittiesOwned::<T>::get(&to);
			to_owned.push(dna.clone());
			kitty.owner = to.clone();

			//updates to storage
			Kitties::<T>::insert(&dna, kitty.clone());
			KittiesOwned::<T>::insert(&to, to_owned);
			KittiesOwned::<T>::insert(&from, from_owned);
			// log::info!("transfer successful: from {:?} to {:?}", &from, &to);

			Self::deposit_event(Event::KittyTransferred{from, to, kitty: kitty.name , dna });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn random_number(_origin: OriginFor<T>) -> DispatchResult {
			// anyone can random
			let (_random_value, block_number) = T::RandomProvider::random(&b"seed sample"[..]);
			let random_info = RandomInfo::<T> {
				hash: _random_value,
				block_number,
			};
			RandomNumber::<T>::put(random_info);
			Ok(())
		}
	}
}

impl<T:Config> Pallet<T> {
	fn calculate_gender(dna: Vec<u8>) -> Result<Gender,Error<T>>{
		let mut res = Gender::Female;
		if dna.len() % 2 ==0 {
			res = Gender::Male;
		}
		Ok(res)
	}

	fn gen_dna(kitty_name: Vec<u8>) -> T::Hash {
		let (seed,block) = T::RandomProvider::random(&kitty_name);
		let block_number = <frame_system::Pallet<T>>::block_number();
		log::info!("seed: {:?}, block:{:?},  block_number: {:?}", seed, block, block_number);
		T::Hashing::hash_of(&(seed, block_number))
	}

	// Use nonce in randomness implementation
	// fn get_and_increment_nonce() -> Vec<u8> {
	// 	let nonce = Nonce::<T>::get();
	// 	Nonce::<T>::put(nonce.wrapping_add(1));
	// 	nonce.encode()
	// }
}
