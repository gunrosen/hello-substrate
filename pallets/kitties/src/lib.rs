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
use frame_support::dispatch::fmt;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type MomentOf<T> =  <<T as Config>::TimeProvider as Time>::Moment;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	// use std;
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T:Config> {
		dna: Vec<u8>,
		owner: T::AccountId,
		price: BalanceOf<T>,
		gender: Gender,
		created_date: MomentOf<T>,
	}
	impl<T:Config> fmt::Debug for Kitty<T>{
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
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
	pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Kitty<T>, OptionQuery>;

	// Store ownership of account Id to kittes
	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyCreated { kitty: Vec<u8>, owner: T::AccountId },
		KittyTransferred { from: T::AccountId, to: T::AccountId, kitty:Vec<u8> },
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
			let gender = Self::calculate_gender(&dna)?;
			let kitty = Kitty::<T> { dna: dna.clone(), price: 0u32.into(), gender, owner: who.clone(), created_date: T::TimeProvider::now()  };
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
			Self::deposit_event(Event::KittyCreated{kitty: dna, owner: who.clone()});
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Put number into storage map
		#[pallet::weight(47_000_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn transfer_kitty_to_friend(origin: OriginFor<T>, to: T::AccountId, dna: Vec<u8>) -> DispatchResult{
			let from = ensure_signed(origin)?;
			let mut kitty = Kitties::<T>::get(&dna).ok_or(Error::<T>::KittyNotFound)?;
			ensure!(kitty.owner == from, Error::<T>::KittyWrongOwner);
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
			Kitties::<T>::insert(&dna, kitty);
			KittiesOwned::<T>::insert(&to, to_owned);
			KittiesOwned::<T>::insert(&from, from_owned);
			// log::info!("transfer successful: from {:?} to {:?}", &from, &to);

			Self::deposit_event(Event::KittyTransferred{from, to, kitty: dna });
			Ok(())
		}
	}
}

impl<T> Pallet<T> {
	fn calculate_gender(dna: &Vec<u8>) -> Result<Gender,Error<T>>{
		let mut res = Gender::Female;
		if dna.len() % 2 ==0 {
			res = Gender::Male;
		}
		Ok(res)
	}
}
