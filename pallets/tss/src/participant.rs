use frame_support::StorageValue;
use codec::{Encode,Decode, Codec, EncodeLike};
use sp_std::prelude::*;

#[derive(Decode, Encode, Clone, Default)]
pub struct Data<T> {
    pub data: Vec<T>,
}

pub trait DataT{
    type AccountType;
    fn get(&self) -> &Self::AccountType;
    fn mut_get(&mut self) -> &mut Self::AccountType;
}

impl <T> DataT for Data<T> {
    type AccountType = Vec<T>;
    fn get(&self) -> & Self::AccountType {
        &self.data
    }

    fn mut_get(&mut self) -> &mut Self::AccountType {
        &mut self.data
    }
}

pub trait AccountCollection {
    type AccountSet;
}

pub trait OptionT {
    type OptionType;
    fn data(&self) -> Option<&Self::OptionType>;
    fn mut_data(&mut self) -> Option<&mut Self::OptionType>;
}

impl  <T>OptionT for Option<Data<T>>{
    type OptionType = Data<T>;
    fn data(&self) -> Option<&Data<T>> {
        match self {
            None => None,
            Some(ref i) => Some(i),
        }
    }

    fn mut_data(&mut self) -> Option<&mut Data<T>> {
        match self {
            None => None,
            Some(ref mut i) => Some(i),
        }
    }
}

impl<T:Codec + PartialEq> Data<T> {
    pub fn new(data: T) -> Data<T> {
        Data::<T> {
            data: vec![data],
        }
    }

    pub fn add_account<C: AccountCollection>(account_data: T)
        where
            C::AccountSet: StorageValue<Data<T>>,
            <C::AccountSet as StorageValue<Data<T>>>::Query:
            OptionT<OptionType=Data<T>>,
            T: Codec + EncodeLike + Clone + Eq + PartialEq + Default,
    {
        /*
        C::AccountSet::mutate(|account_vec| {
            if let Some(ref mut data) = account_vec.mut_data() {
                data.mut_get().push(account_data);
            }else {
                let new_data = Self::new(account_data);
                C::AccountSet::put(new_data);
            }
        });
        */
        let mut account_vec = C::AccountSet::get();
        match account_vec.mut_data(){
            Some(data) => { data.mut_get().push(account_data); },
            None => {  let new_data = Self::new(account_data);
                C::AccountSet::put(new_data); }
        }

    }

    pub fn remove_account<C: AccountCollection>(account_data: T) where
        C::AccountSet: StorageValue<Data<T>>,
        <C::AccountSet as StorageValue<Data<T>>>::Query:
        OptionT<OptionType=Data<T>>,
        T: Codec + EncodeLike + Clone + Eq + PartialEq + Default,
    {
        C::AccountSet::mutate(|account_vec| {
            if let Some(ref mut data) = account_vec.mut_data() {
                let mut index = 999;
                data.mut_get().into_iter().enumerate().for_each(|(i, each_account)| {
                    if each_account == &account_data {
                        index = i;
                    }
                });
                if index != 999 {
                    data.mut_get().remove(index);
                }
            }
        });
    }

    pub fn accessible<C: AccountCollection>(account_data: T) -> bool
        where
            C::AccountSet: StorageValue<Data<T>>,
            <C::AccountSet as StorageValue<Data<T>>>::Query:
            OptionT<OptionType=Data<T>>,
            T: Codec + EncodeLike + Clone + Eq + PartialEq + Default,
    {
        let mut account_vec = C::AccountSet::get();
        if let Some(ref mut data) = account_vec.mut_data() {
            let account_vec = data.mut_get();
            return account_vec.contains(&account_data);
        } else {
            return false;
        }
    }

    pub fn initdata<C: AccountCollection>(&self)
        where
            C::AccountSet: StorageValue<Data<T>>,
            T: Codec + EncodeLike + Clone + Eq + PartialEq + Default,
    {
        C::AccountSet::put(self);
    }

}





