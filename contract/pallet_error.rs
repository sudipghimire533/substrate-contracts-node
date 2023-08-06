#[derive(Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Debug))]
#[repr(u8)]
pub enum Error {
	BalanceLow = 1,
	NoAccount = 2,
	NoPermission = 3,
	Unknown = 4,
	Frozen = 5,
	InUse = 6,
	BadWitness = 7,
	MinBalanceZero = 8,
	UnavailableConsumer = 9,
	BadMetadata = 10,
	Unapproved = 11,
	WouldDie = 12,
	AlreadyExists = 13,
	NoDeposit = 14,
	WouldBurn = 15,
	LiveAsset = 16,
	AssetNotLive = 17,
	IncorrectStatus = 18,
	NotFrozen = 19,
	CallbackFailed = 20,
}

impl TryFrom<u8> for Error {
	type Error = ();

	fn try_from(code: u8) -> Result<Error, Self::Error> {
		match code {
			1 => Ok(Error::BalanceLow),
			2 => Ok(Error::NoAccount),
			3 => Ok(Error::NoPermission),
			4 => Ok(Error::Unknown),
			5 => Ok(Error::Frozen),
			6 => Ok(Error::InUse),
			7 => Ok(Error::BadWitness),
			8 => Ok(Error::MinBalanceZero),
			9 => Ok(Error::UnavailableConsumer),
			10 => Ok(Error::BadMetadata),
			11 => Ok(Error::Unapproved),
			12 => Ok(Error::WouldDie),
			13 => Ok(Error::AlreadyExists),
			14 => Ok(Error::NoDeposit),
			15 => Ok(Error::WouldBurn),
			16 => Ok(Error::LiveAsset),
			17 => Ok(Error::AssetNotLive),
			18 => Ok(Error::IncorrectStatus),
			19 => Ok(Error::NotFrozen),
			20 => Ok(Error::CallbackFailed),
			_ => Err(()),
		}
	}
}
