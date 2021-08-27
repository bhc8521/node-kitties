#![cfg_attr(not(feature = "std"), no_std)]



pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{Randomness}};
	use frame_system::{pallet, pallet_prelude::*};
	use codec::{Encode, Decode};
	use transfer


	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8;16]);

	type KittyIndex = u32;

	#[pallet::config]
	pub trait Config: pallet_balances::Config + frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	


	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<T::AccountId>, ValueQuery>;


	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, KittyIndex),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameParentIndex,
		InvalidKittyIndex,
		SameOwner,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => {
					1
				} 
			};
			
			let dna = Self::random_value(&who);
			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id + 1);

			Self::deposit_event(Event::KittyCreate(who, kitty_id));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::owner(kitty_id).unwrap() == who.clone(), Error::<T>::NotOwner);
			Owner::<T>::insert(kitty_id, Some(new_owner.clone()));
			Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);
			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => {
					1
				} 
			};

			let dna1 = kitty1.0;
			let dna2 = kitty2.0;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8;16];
			for i in 0..dna1.len() {
				new_dna[i] = (selector[i] & dna1[i]) | (selector[i] & dna2[i]);
			}

			Kitties::<T>::insert(kitty_id, Some(Kitty(new_dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id + 1);

			Self::deposit_event(Event::KittyCreate(who, kitty_id));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn buy(origin: OriginFor<T>, seller: T::AccountId, kitty_id: KittyIndex, amount: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(seller == Self::owner(kitty_id).unwrap(), Error::<T>::NotOwner);
			ensure!(who != seller, Error::<T>::SameOwner);
			T::transfer_keep_alive(&who, &seller, amount);
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn sell(origin: OriginFor<T>, buyer: T::AccountId, kitty_id: KittyIndex, amount: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == Self::owner(kitty_id).unwrap(), Error::<T>::NotOwner);
			ensure!(who != buyer, Error::<T>::SameOwner);
			pallet_balances::Pallet::transfer_keep_alive(&buyer, &who, amount);
			Owner::<T>::insert(kitty_id, Some(buyer.clone()));
			Ok(())
		}
		
	}

	impl<T: Config> Pallet<T> { 
		fn random_value(sender: &T::AccountId) -> [u8;16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(sp_io::hashing::blake2_128)
		}
	}
}
