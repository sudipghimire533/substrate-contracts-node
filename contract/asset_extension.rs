pub type AssetAmount = u128;
pub type AssetId = u32;
pub type AccountId = <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId;

#[derive(Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Debug))]
pub enum ExtensionError {
	/// Dispatch Error
	DispatchError,
	/// The error is dispatch error but is error from
	/// pallet-assets, so it is even better to elaborate the error
	PalletError(crate::pallet_error::Error),
	/// Error while encoding
	EncodingError,
	/// Unknown status code
	Other(u32),
}

impl ink::env::chain_extension::FromStatusCode for ExtensionError {
	fn from_status_code(status_code: u32) -> Result<(), Self> {
		match status_code {
			0 => Ok(()),
			200 => Err(Self::DispatchError),
			a => match crate::pallet_error::Error::try_from(status_code as u8) {
				Ok(e) => Err(Self::PalletError(e)),
				Err(_) => Err(Self::Other(a)),
			},
		}
	}
}

impl From<scale::Error> for ExtensionError {
	fn from(_: scale::Error) -> Self {
		Self::EncodingError
	}
}

#[ink::chain_extension]
pub trait AssetExtension {
	type ErrorCode = ExtensionError;

	#[ink(extension = 120)]
	fn transfer_keep_alive(
		asset_id: AssetId,
		caller: AccountId,
		target: AccountId,
		amount: AssetAmount,
	) -> Result<(), ExtensionError>;

	#[ink(extension = 121)]
	fn transfer_allow_death(
		asset_id: AssetId,
		caller: AccountId,
		target: AccountId,
		amount: AssetAmount,
	) -> Result<(), ExtensionError>;

	#[ink(extension = 122)]
	fn approve_transfer(
		asset_id: AssetId,
		caller: AccountId,
		delegate: AccountId,
		amount: AssetAmount,
	) -> Result<(), ExtensionError>;

	#[ink(extension = 123)]
	fn cancel_approval(
		asset_id: AssetId,
		caller: AccountId,
		delegate: AccountId,
	) -> Result<(), ExtensionError>;

	#[ink(extension = 124)]
	fn transfer_approved(
		asset_id: AssetId,
		caller: AccountId,
		owner: AccountId,
		target: AccountId,
		amount: AssetAmount,
	) -> Result<(), ExtensionError>;
}
