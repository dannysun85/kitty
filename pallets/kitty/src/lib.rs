#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::traits::Randomness;
    use frame_system::pallet_prelude::*;
    use sp_io::hashing::blake2_128;

    type KittyIndex = u32;

    #[pallet::type_value]
    pub fn GetDefaultValue() -> KittyIndex {
        0_u32
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct Kitty(pub [u8;16]);


    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_kitty_id)]
    pub type NextKittyId<T> = StorageValue<_,KittyIndex, ValueQuery, GetDefaultValue>;

    #[pallet::storage]
    #[pallet::getter(fn kittys)]
    pub type Kittys<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Kitty>;

    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreated(T::AccountId, KittyIndex, Kitty),
        KittyBred(T::AccountId, KittyIndex, Kitty),
        KittyTransferred(T::AccountId, T::AccountId, KittyIndex),
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidKittyId,
        SameKittyId,
        NotOwner,
    }


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create(origin: OriginFor<T>) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);

            Kittys::<T>::insert(kitty_id, &kitty);
            KittyOwner::<T>::insert(kitty_id, &sender);
            NextKittyId::<T>::put(kitty_id + 1);

            Self::deposit_event(Event::KittyCreated(sender, kitty_id, kitty));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn breed(origin:OriginFor<T>, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) -> DispatchResult{
            let sender = ensure_signed(origin)?;

            //check kitty id
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
            let kitty1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
            let kitty2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;

            //get next id
            let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

            //selector for breeding
            let selector = Self::random_value(&sender);

            let mut data = [0u8; 16];
            for i in 0..kitty1.0.len(){
                data[i] = (kitty1.0[i] & selector[i]) | (kitty2.0[i] & !selector[i]);
            }

            let new_kitty = Kitty(data);

            Kittys::<T>::insert(kitty_id, &new_kitty);
            KittyOwner::<T>::insert(kitty_id, &sender);
            NextKittyId::<T>::put(kitty_id + 1);

            Self::deposit_event(Event::KittyBred(sender, kitty_id, new_kitty));
            Ok(())

        }

        #[pallet::weight(10_000)]
        pub fn transfer(origin:OriginFor<T>, kitty_id:KittyIndex, new_owner: T::AccountId) -> DispatchResult{
            let sender = ensure_signed(origin)?;

            //check kitty id
            let _id = Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

            //check owner
            ensure!(Self::kitty_owner(kitty_id) == Some(sender.clone()), Error::<T>::NotOwner);

            //transfer
            KittyOwner::<T>::insert(kitty_id, &new_owner);

            Self::deposit_event(Event::KittyTransferred(sender, new_owner,kitty_id));

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn buy_from(origin:OriginFor<T>, kitty_id:KittyIndex, price:u128,buyer: T::AccountId) ->DispatchResult{}

        #[pallet::weight(10_000)]
        pub fn sell_for(origin:OriginFor<T>, kitty_id:KittyIndex, price:u128, seller: T::AccountId) ->DispatchResult{}
    }

    impl<T:Config> Pallet<T>{
        // get  random 256
        fn random_value(sender: &T::AccountId) -> [u8;16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );
            payload.using_encoded(blake2_128)
        }

        // get next id
        fn get_next_id() -> Result<KittyIndex,()>{
            match Self::next_kitty_id() {
                KittyIndex::MAX => Err(()),
                val => Ok(val),
            }
        }

        //get kitty via id
        fn get_kitty(kitty_id:KittyIndex) -> Result<Kitty,()>{
            match Self::kittys(kitty_id) {
                Some(kitty) => Ok(kitty),
                None => Err(()),
            }
        }
    }



}