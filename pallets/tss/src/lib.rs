#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch::DispatchResult, traits::Get};
use frame_system::ensure_signed;
use frame_support::dispatch::Vec;
use codec::{Decode, Encode};
use pallet_timestamp;
use frame_system::ensure_root;

use sp_std::{ prelude::*, marker::PhantomData};
use frame_support::sp_runtime::RuntimeDebug;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod participant;
pub use participant::*;

#[derive(PartialEq, Eq, Clone)]
pub enum TssKeyType {
	BTC,
	FileCoin,
	Bool,
	Normal,
}

/// for withdraw event
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct WithdrawDetail<AccountId> {
	pub uid: u64,
	pub actor: AccountId,
	/// token name
	pub token: Vec<u8>,
	pub value: u128,
	pub receiver: Vec<u8>,
}

pub struct LinkedNodes<T: Trait>(PhantomData<T>);

impl<T: Trait> AccountCollection for LinkedNodes<T> {
	type AccountSet = VerifiedAccount<T>;
}

pub trait Trait: frame_system::Trait + pallet_timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {

	    Index get(fn index): u64 ;
        Members get(fn members): Vec<T::AccountId>;
        // BTC part
        // Combine of public key for the TSS pubkey
        TssPubKey get(fn tss_pubkey): Vec<u8>;

        // FileCoin part
        // Combine of public key for the TSS pubkey
        TssPubKeyFC get(fn tss_pubkey_fc): Vec<u8>;

        // Bool part
        // Combine of public key for the TSS pubkey
        TssPubKeyBool get(fn tss_pubkey_bool): Vec<u8>;

        // Fragments of public key for the TSS pubkey
        TssPubKeyVec get(fn tss_pubkey_vec): map hasher(blake2_128_concat) Vec<u8> => Vec<Vec<u8>>;

        IsCreated get(fn is_created): bool = false;
        IsCreating get(fn is_creating): bool = false;
        TssUrl get(fn tss_url): Vec<u8>;

        pub VerifiedAccount get(fn verified_account): Option<Data<T::AccountId>>;

        FileCoinToken get(fn file_coin_token): map hasher(blake2_128_concat) T::AccountId => u128;
	}
}

decl_event!(
	pub enum Event<T>
	    where
	             AccountId = <T as frame_system::Trait>::AccountId,
	             Time = <T as pallet_timestamp::Trait>::Moment,

	    {
	        GenKey(u64, AccountId, Time, Vec<u8>), // url
            SignMessage(u64, AccountId, Time, Vec<u8>, Vec<u8>, Vec<u8>), // url message pubkey
            GenSuccess(u64, AccountId, Time),
            SignSuccess(u64, u64, Time),

            GenerateTssKey(Vec<u8>,Vec<u8>), // url store
            GenerateTssKeyBool(Vec<u8>,Vec<u8>), // url store
            GenerateTssKeyFc(Vec<u8>,Vec<u8>), // url store

            SignBtcMessage(u64, Time, Vec<u8>, Vec<u8>, Vec<u8>), // url btc_tx_message(hex) pubkey

            //withdraw event
            WithdrawToken(WithdrawDetail<AccountId>),
     	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		//type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

	     /// start to create a tss key pair
        #[weight = 0]
        fn key_gen(origin, url:Vec<u8>,store:Vec<u8>) -> DispatchResult{
            //let _sender = ensure_root(origin)?;
            Self::gen_key(url,store)
        }


        #[weight = 0]
        fn key_gen_bool(origin, url:Vec<u8>,store:Vec<u8>) -> DispatchResult{
            //let _sender = ensure_root(origin)?;
            Self::gen_key_bool(url,store)
        }

        #[weight = 0]
        fn key_gen_fc(origin, url:Vec<u8>,store:Vec<u8>) -> DispatchResult{
            //let _sender = ensure_root(origin)?;
            Self::gen_key_fc(url,store)
        }

        #[weight = 0]
        fn gen_key_set_false(origin) -> DispatchResult{
            let _sender = ensure_signed(origin)?;
            Self::gen_key_false();
            Ok(())
        }

        // tss key pair result
        #[weight = 0]
        fn key_created_result_is(origin,pubkey:Vec<u8>,pubkey_vec:Vec<Vec<u8>>,store:Vec<u8>) -> DispatchResult {
            //let _ = ensure_root(origin)?;
            Self::key_created_result(pubkey,pubkey_vec,store,TssKeyType::BTC)
        }

        // tss key pair result for bool
        #[weight = 0]
        fn key_created_result_is_bool(origin,pubkey:Vec<u8>,pubkey_vec:Vec<Vec<u8>>,store:Vec<u8>) -> DispatchResult {
            //let _ = ensure_root(origin)?;
            Self::key_created_result(pubkey,pubkey_vec,store,TssKeyType::Bool)
        }

        // tss key pair result for fc
        #[weight = 0]
        fn key_created_result_is_fc(origin,pubkey:Vec<u8>,pubkey_vec:Vec<Vec<u8>>,store:Vec<u8>) -> DispatchResult {
            //let _ = ensure_root(origin)?;
            Self::key_created_result(pubkey,pubkey_vec,store,TssKeyType::FileCoin)
        }

        // sign normal message
        #[weight = 0]
        fn sign_message(origin,url:Vec<u8>,message:Vec<u8>,pubkey:Vec<u8>) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            Self::deposit_event(RawEvent::SignMessage
                     (Self::tss_index()
                     , sender
                     , <pallet_timestamp::Module<T>>::get()
                     , url, message,
                     pubkey));
            Ok(())
        }

        #[weight = 0]
        fn sign_success(origin) -> DispatchResult{
            //let sender = ensure_root(origin)?;
            Self::deposit_event(RawEvent::SignSuccess(Self::tss_index()
                                                      , 0u64
                                                      , <pallet_timestamp::Module<T>>::get()));
            Ok(())
        }

        #[weight = 0]
        fn set_tss_url(origin,url:Vec<u8>) -> DispatchResult{
            //let _ = ensure_root(origin)?;
            TssUrl::put(url);
            Ok(())
        }

        // test sign Btc Message
        #[weight = 0]
        fn test_sign(origin,url:Vec<u8>,btc_message:Vec<u8>,pubkey:Vec<u8>) -> DispatchResult{
            let _sender = ensure_signed(origin)?;

            Self::deposit_event(RawEvent::SignBtcMessage
                (Self::tss_index(), <pallet_timestamp::Module<T>>::get()
                , url, btc_message, pubkey));

            Ok(())
        }

        #[weight = 0]
        pub fn add_new_one(origin, id:T::AccountId) -> DispatchResult{
            let _sender = ensure_root(origin)?;
            Self::add_new_member(id);
            Ok(())
        }

        #[weight = 0]
        pub fn test_check_permissions(origin) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            let result1 = Self::check_permissions(sender.clone());

            Self::add_new_member(sender.clone());
            let result2 = Self::check_permissions(sender);

            Ok(())
        }

        #[weight = 0]
        pub fn deposit_token(origin, who:T::AccountId, amount_add:u128) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            let current_balance = <FileCoinToken<T>>::get(who.clone());
            <FileCoinToken<T>>::insert(who,current_balance + amount_add);
            Ok(())
        }

        #[weight = 0]
        pub fn withdraw_token(origin, who:T::AccountId, amount_add:u128) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            let current_balance = <FileCoinToken<T>>::get(who.clone());
            <FileCoinToken<T>>::insert(who,current_balance - amount_add);
            Ok(())
        }


    }
}


impl<T: Trait> Module<T> {
	fn tss_index() -> u64 {
		let index_old = Index::get() + 1;
		Index::put(index_old);
		index_old
	}

	pub fn sign_btc_tx(url:Vec<u8>,btc_message:Vec<u8>,pubkey:Vec<u8>) -> DispatchResult{
		#[allow(unused_assignments)]
			let mut final_url = vec![0u8];
		if url == vec![0u8] {
			final_url = TssUrl::get();
		}else {
			final_url = url;
		}
		let mut final_pubkey = vec![0u8];
		if final_pubkey == vec![0u8] {
			final_pubkey = TssPubKey::get();
		}else {
			final_pubkey = pubkey;
		}

		Self::deposit_event(RawEvent::SignBtcMessage
			(Self::tss_index(),  <pallet_timestamp::Module<T>>::get()
			 , final_url, btc_message,final_pubkey));

		Ok(())
	}

	pub fn gen_key(url:Vec<u8>,store:Vec<u8>) -> DispatchResult{
		Self::deposit_event(RawEvent::GenerateTssKey(url,store));
		Ok(())
	}

	pub fn gen_key_bool(url:Vec<u8>,store:Vec<u8>) -> DispatchResult{
		Self::deposit_event(RawEvent::GenerateTssKeyBool(url,store));
		Ok(())
	}

	pub fn gen_key_fc(url:Vec<u8>,store:Vec<u8>) -> DispatchResult{
		Self::deposit_event(RawEvent::GenerateTssKeyFc(url,store));
		Ok(())
	}

	pub fn gen_key_false(){
		IsCreated::put(false);
		IsCreating::put(false);
	}

	pub fn key_created_result(pubkey:Vec<u8>, pubkey_vec:Vec<Vec<u8>>, _store:Vec<u8>, keytype: TssKeyType) -> DispatchResult{

		match keytype {
			TssKeyType::BTC  => { TssPubKey::put(pubkey.clone()); },
			TssKeyType::Bool => { TssPubKeyBool::put(pubkey.clone()); },
			TssKeyType::FileCoin => { TssPubKeyFC::put(pubkey.clone()); },
			TssKeyType::Normal => { },

		}
		TssPubKeyVec::insert(pubkey,pubkey_vec);

		Ok(())
	}

	pub fn check_permissions(id: T::AccountId) -> DispatchResult {
		match Data::accessible::<LinkedNodes<T>>(id){
			true => Ok(()),
			false =>Err(Error::<T>::NoneValue)? ,
		}
	}

	pub fn add_new_member(id:T::AccountId){
		Data::add_account::<LinkedNodes<T>>(id);
	}

	pub fn delete_member(id:T::AccountId){
		Data::remove_account::<LinkedNodes<T>>(id);
	}

}