use super::Runtime;
use codec::Encode;
use frame_support::log::{error, trace};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
	BufIn, BufInBufOutState, ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{DispatchError, MultiAddress::Id as AddressId};

const TRANSFER_KEEP_ALIVE: u16 = 120;
const TRANSFER_ALLOW_DEATH: u16 = 121;
const APPROVE_TRANSFER: u16 = 122;
const CANCEL_APPROVED: u16 = 123;
const TRANSFER_APPROVED: u16 = 124;

/// StatusCode to use when DispatchError is other than of
/// pallet-asset
// i.e
// this also means that if pallet-error throws some error
// from pallet-balance, we would consider it as the generic error
// for shake of simplicity
const DISPATCH_ERROR_GENERIC: u32 = 200;
/// The offset to converge asset-pallet error to
/// Final status_code witll be PALLET_ERROR_OFFSET + ERROR INDEX
const PALLET_ERROR_OFFSET: u32 = 1;
/// Status code that signify success
const SUCCESS: u32 = 0;
/// Module Index of pallet-asset
// check in crate::construct_runtime!
const PALLET_ASSET_INDEX: u8 = 9;

/// Contract extension for `FetchRandom`
#[derive(Default)]
pub struct AssetExtension;

impl AssetExtension {
	/// Convert the Result<(), DispatchError> to RetVal
	/// This has to be in sync with conversion implemented in contract/
	///
	/// Basic idea is to return 0 for success
	/// status code from 1.. for error from paller-asset;
	/// status code 200 for every other error
	pub fn get_retval(pallet_res: Result<(), DispatchError>) -> RetVal {
		match pallet_res {
			Ok(_) => RetVal::Converging(SUCCESS),
			Err(err) => {
				match err {
					DispatchError::Module(err) if err.index == PALLET_ASSET_INDEX => {
						// The pallet error type is basically [u8; 4]
						// but in pallet asset all error count fit into u8,
						// so
						let error_index = err.error[0];
						RetVal::Converging((1 + error_index).into())
					},
					_ => RetVal::Converging(DISPATCH_ERROR_GENERIC),
				}
			},
		}
	}
}

impl ChainExtension<Runtime> for AssetExtension {
	/// Once the chain_extension data is received from
	/// ink runtime,
	/// logic to handle
	// remember: everything is encoded
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		type AssetId = <Runtime as pallet_assets::Config>::AssetId;
		type AssetAmount = <Runtime as pallet_assets::Config>::Balance;
		type AccountId = <Runtime as frame_system::Config>::AccountId;

		let func_id = env.func_id();
		let mut env_buf = env.buf_in_buf_out();

		match func_id {
			// transfer balance from caller to target
			// making sure the asset balance is >= to existinsial deposit
			// of asset i.e keep account alive
			TRANSFER_KEEP_ALIVE => {
				let (asset_id, caller, target, amount): (
					AssetId,
					AccountId,
					AccountId,
					AssetAmount,
				) = env_buf.read_as()?;
				let origin = RawOrigin::Signed(caller);

				let pallet_res = pallet_assets::Pallet::<Runtime>::transfer_keep_alive(
					origin.into(),
					asset_id.into(),
					target.into(),
					amount.into(),
				);

				Ok(AssetExtension::get_retval(pallet_res))
			},

			// transfer given amount from caller to target
			// if this transfer result in caller account to be dropped
			// do it anyway
			TRANSFER_ALLOW_DEATH => {
				let (asset_id, caller, target, amount): (
					AssetId,
					AccountId,
					AccountId,
					AssetAmount,
				) = env_buf.read_as()?;
				let origin = RawOrigin::Signed(caller);

				let pallet_res = pallet_assets::Pallet::<Runtime>::transfer(
					origin.into(),
					asset_id.into(),
					target.into(),
					amount.into(),
				);

				Ok(AssetExtension::get_retval(pallet_res))
			},

			// approve delegated transfer call
			APPROVE_TRANSFER => {
				let (asset_id, caller, delegate, amount): (
					AssetId,
					AccountId,
					AccountId,
					AssetAmount,
				) = env_buf.read_as()?;
				let origin = RawOrigin::Signed(caller);

				let pallet_res = pallet_assets::Pallet::<Runtime>::approve_transfer(
					origin.into(),
					asset_id.into(),
					delegate.into(),
					amount.into(),
				);

				Ok(AssetExtension::get_retval(pallet_res))
			},

			// cancel approval
			CANCEL_APPROVED => {
				let (asset_id, caller, delegate): (AssetId, AccountId, AccountId) =
					env_buf.read_as()?;
				let origin = RawOrigin::Signed(caller);

				let pallet_res = pallet_assets::Pallet::<Runtime>::cancel_approval(
					origin.into(),
					asset_id.into(),
					delegate.into(),
				);

				Ok(AssetExtension::get_retval(pallet_res))
			},

			// transfer with approval
			TRANSFER_APPROVED => {
				let (asset_id, caller, delegate, amount): (
					AssetId,
					AccountId,
					AccountId,
					AssetAmount,
				) = env_buf.read_as()?;
				let origin = RawOrigin::Signed(caller.clone());

				let pallet_res = pallet_assets::Pallet::<Runtime>::transfer_approved(
					origin.into(),
					asset_id.into(),
					caller.into(),
					delegate.into(),
					amount.into(),
				);
				Ok(AssetExtension::get_retval(pallet_res))
			},

			// We don't know what to do.
			//
			// make sure that all function in contract/asset_extension.rs
			// are indexed properly
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"))
			},
		}
	}

	// we enable the Extension,
	// so we can upload the contract which access chain_extension
	// externalities
	fn enabled() -> bool {
		true
	}
}
