#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod asset_extension;
pub mod pallet_error;

use asset_extension::{AccountId, AssetAmount, AssetId, ExtensionError};
use ink::env::Environment;

/// Environment to import into contract
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

/// All the Environment configuration of CostumEnvironment
/// is same as DefaultEnvironment with just added ChainExtension
impl Environment for CustomEnvironment {
	type AccountId = AccountId;
	type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
	type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
	type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
	type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;
	const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

	type ChainExtension = asset_extension::AssetExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod asset_ext {
	use super::*;

	/// Defines the storage of our contract.
	///
	/// Here we store the random seed fetched from the chain.
	#[ink(storage)]
	pub struct AssetExtension {
		/// The asset id this contract is supposed to interact with
		// NOTE:
		// we can try to make this contract generic over all assets
		// but in real production all asset serves different purpose and a single
		// contract to interact with will of them might not be ideal case
		pub asset_id: u32,
	}

	impl AssetExtension {
		/// Constructor that initializes the `bool` value to the given `init_value`.
		#[ink(constructor)]
		pub fn new(asset_id: u32) -> Self {
			Self { asset_id }
		}

		/// Get the AssetId this contract is interfacing to
		#[ink(message)]
		pub fn asset_id(&self) -> u32 {
			self.asset_id
		}

		/// Transfer the asset from caller of contract
		/// to target.
		/// Transaction is of amount `amount`
		#[ink(message)]
		pub fn transfer_keep_alive(
			&self,
			target: AccountId,
			amount: AssetAmount,
		) -> Result<(), ExtensionError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env()
				.extension()
				.transfer_keep_alive(asset_id, sender, target, amount)
				.map(|_| ())
		}

		/// Transfer all the asset from caller to target
		#[ink(message)]
		pub fn transfer_allow_death(
			&self,
			target: AccountId,
			amount: AssetAmount,
		) -> Result<(), ExtensionError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env()
				.extension()
				.transfer_allow_death(asset_id, sender, target, amount)
				.map(|_| ())
		}

		/// Approve an amount of asset for transfer by a delegated third-party account.
		#[ink(message)]
		pub fn approve_transfer(
			&self,
			target: AccountId,
			amount: AssetAmount,
		) -> Result<(), ExtensionError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env()
				.extension()
				.approve_transfer(asset_id, sender, target, amount)
				.map(|_| ())
		}

		/// Cancel all of some asset approved for delegated transfer by a third-party account.
		#[ink(message)]
		pub fn cancel_approval(&self, target: AccountId) -> Result<(), ExtensionError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env().extension().cancel_approval(asset_id, sender, target).map(|_| ())
		}

		/// Transfer some asset balance from a previously delegated account to some third-party
		/// account.
		#[ink(message)]
		pub fn transfer_approved(
			&self,
			owner: AccountId,
			destination: AccountId,
			amount: AssetAmount,
		) -> Result<(), ExtensionError> {
			let caller = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env()
				.extension()
				.transfer_approved(asset_id, caller, owner, destination, amount)
				.map(|_| ())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;
		use boilerplate_rs::*;
		use scale::Encode;

		const ENV_CALLER_ACCOUNT: [u8; 32] = [1_u8; 32];
		const ASSET_ID: AssetId = 1_u32;

		fn new_account() -> AccountId {
			AccountId::from([rand::random(); 32])
		}

		#[derive(Default)]
		pub struct MockedExtension {
			pub func_id: u32,
			pub logic: Option<Box<dyn Fn(&MockedExtension, &[u8], &mut Vec<u8>) -> u32>>,
		}

		impl ink::env::test::ChainExtension for MockedExtension {
			fn func_id(&self) -> u32 {
				self.func_id
			}

			fn call(&mut self, input: &[u8], output: &mut Vec<u8>) -> u32 {
				match &self.logic {
					Some(logic) => logic(&*self, input, output),
					None => panic!("No executor"),
				}
			}
		}

		#[ink::test]
		fn transfer_asset() {
			let mut asset_extension = AssetExtension::new(ASSET_ID);
			let target = AccountId::from([20; 32]);
			let amount = 100_u128;

			let execute = move |ext: &MockedExtension, input: &[u8], output: &mut Vec<u8>| -> u32 {
				let mut expected_input: Vec<u8> = vec![];

				expected_input.append(&mut ASSET_ID.encode());
				expected_input.append(&mut ENV_CALLER_ACCOUNT.encode());
				expected_input.append(&mut target.encode());
				expected_input.append(&mut amount.encode());

				let input_range = (input.len() - expected_input.len())..input.len();
				assert_eq!(input[input_range], expected_input);

				0
			};

			ink::env::test::register_chain_extension(MockedExtension {
				func_id: 120,
				logic: Some(Box::new(execute)),
			});
			assert_eq!(asset_extension.transfer_keep_alive(target, amount), Ok(()));
		}

		#[ink::test]
		fn transfer_error() {
			let mut asset_extension = AssetExtension::new(ASSET_ID);

			let execute =
				move |ext: &MockedExtension, input: &[u8], output: &mut Vec<u8>| -> u32 { 6 };

			ink::env::test::register_chain_extension(MockedExtension {
				func_id: 120,
				logic: Some(Box::new(execute)),
			});
			assert_eq!(
				asset_extension.transfer_keep_alive(AccountId::from([10; 32]), 100),
				Err(crate::asset_extension::ExtensionError::PalletError(
					crate::pallet_error::Error::InUse
				))
			);
		}

		#[ink::test]
		fn all_extension_call_pass() {
			let mut asset_extension = AssetExtension::new(ASSET_ID);

			let execute =
				move |ext: &MockedExtension, input: &[u8], output: &mut Vec<u8>| -> u32 { 0 };

			for func_id in [120, 121, 122, 123, 124] {
				ink::env::test::register_chain_extension(MockedExtension {
					func_id,
					logic: Some(Box::new(execute)),
				});
			}

			let acc_a = AccountId::from([10; 32]);
			let acc_b = AccountId::from([20; 32]);
			let amount = 100_u128;

			assert_ok!(asset_extension.cancel_approval(acc_a));
			assert_ok!(asset_extension.approve_transfer(acc_a, amount));
			assert_ok!(asset_extension.transfer_keep_alive(acc_a, amount));
			assert_ok!(asset_extension.transfer_allow_death(acc_a, amount));
			assert_ok!(asset_extension.transfer_approved(acc_a, acc_b, amount));
		}

		#[ink::test]
		fn all_extension_call_fail() {
			const STATUS_CODE: u32 = 70;
			let mut asset_extension = AssetExtension::new(ASSET_ID);

			let execute = move |ext: &MockedExtension, input: &[u8], output: &mut Vec<u8>| -> u32 {
				STATUS_CODE
			};

			for func_id in [120, 121, 122, 123, 124] {
				ink::env::test::register_chain_extension(MockedExtension {
					func_id,
					logic: Some(Box::new(execute)),
				});
			}

			let acc_a = AccountId::from([10; 32]);
			let acc_b = AccountId::from([20; 32]);
			let amount = 100_u128;
			let err = crate::asset_extension::ExtensionError::Other(STATUS_CODE);

			assert_err_eq!(asset_extension.cancel_approval(acc_a), err);
			assert_err_eq!(asset_extension.approve_transfer(acc_a, amount), err);
			assert_err_eq!(asset_extension.transfer_keep_alive(acc_a, amount), err);
			assert_err_eq!(asset_extension.transfer_allow_death(acc_a, amount), err);
			assert_err_eq!(asset_extension.transfer_approved(acc_a, acc_b, amount), err);
		}
	}
}
