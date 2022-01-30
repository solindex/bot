use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::{convert::TryInto, num::NonZeroU8};

pub const PUBKEY_LENGTH: usize = 32;

pub const BONFIDA_FEE: &str = "31LVSggbVz4VcwBSPdtK8HJ3Lt1cKTJUVQTRNNYMfqBq";
pub const BONFIDA_BNB: &str = "3oQzjfjzUkJ5qHsERk2JPEpAKo34dxAQjUriBqursfxU";

// Pool state is composed of PoolHeader, Array of markets (pubkeys) and array of poolassets

#[derive(Debug, PartialEq)]
pub struct PoolAsset {
    pub mint_address: Pubkey,
}
#[derive(Debug, PartialEq)]
pub enum PoolStatus {
    Uninitialized,
    Unlocked,
    Locked,
    /// Maximum number of pending orders is 64, minimum is 1.
    PendingOrder(NonZeroU8),
    LockedPendingOrder(NonZeroU8),
}

#[derive(Debug, PartialEq)]
pub struct PoolHeader {
    pub serum_program_id: Pubkey,
    pub seed: [u8; 32],
    pub signal_provider: Pubkey,
    pub status: PoolStatus,
    pub number_of_markets: u16,
    pub fee_ratio: u16,
    pub last_fee_collection_timestamp: u64,
    pub fee_collection_period: u64,
}

const STATUS_PENDING_ORDER_FLAG: u8 = 1 << 6;
const STATUS_PENDING_ORDER_MASK: u8 = 0x3f;
const STATUS_LOCKED_FLAG: u8 = 2 << 6;
const STATUS_UNLOCKED_FLAG: u8 = STATUS_PENDING_ORDER_MASK;

impl Sealed for PoolHeader {}

impl Pack for PoolHeader {
    const LEN: usize = 117;

    fn pack_into_slice(&self, target: &mut [u8]) {
        let serum_program_id_bytes = self.serum_program_id.to_bytes();
        target[0..32].copy_from_slice(&serum_program_id_bytes);
        target[32..64].copy_from_slice(&self.seed);
        let signal_provider_bytes = self.signal_provider.to_bytes();
        target[64..96].copy_from_slice(&signal_provider_bytes);
        target[96] = match self.status {
            PoolStatus::Uninitialized => 0,
            PoolStatus::Unlocked => STATUS_UNLOCKED_FLAG,
            PoolStatus::Locked => STATUS_LOCKED_FLAG,
            PoolStatus::PendingOrder(n) => {
                STATUS_PENDING_ORDER_FLAG | (STATUS_PENDING_ORDER_MASK & (n.get() - 1))
            }
            PoolStatus::LockedPendingOrder(n) => {
                STATUS_LOCKED_FLAG
                    | STATUS_PENDING_ORDER_FLAG
                    | (STATUS_PENDING_ORDER_MASK & (n.get() - 1))
            }
        };
        let number_of_markets_bytes = self.number_of_markets.to_le_bytes();
        target[97..99].copy_from_slice(&number_of_markets_bytes);
        target[99..101].copy_from_slice(&self.fee_ratio.to_le_bytes());
        target[101..109].copy_from_slice(&self.last_fee_collection_timestamp.to_le_bytes());
        target[109..117].copy_from_slice(&self.fee_collection_period.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let serum_program_id = Pubkey::new(&src[..32]);
        let seed: [u8; 32] = src[32..64].try_into().unwrap();
        let signal_provider = Pubkey::new(&src[64..96]);
        let status = if src[96] == 0 {
            PoolStatus::Uninitialized
        } else {
            match src[96] >> 6 {
                0 => PoolStatus::Unlocked,
                1 => PoolStatus::PendingOrder(
                    NonZeroU8::new((src[96] & STATUS_PENDING_ORDER_MASK) + 1)
                        .ok_or(ProgramError::InvalidArgument)?,
                ),
                2 => PoolStatus::Locked,
                3 => PoolStatus::LockedPendingOrder(
                    NonZeroU8::new((src[96] & STATUS_PENDING_ORDER_MASK) + 1)
                        .ok_or(ProgramError::InvalidArgument)?,
                ),
                _ => return Err(ProgramError::InvalidAccountData),
            }
        };
        let number_of_markets = u16::from_le_bytes(src[97..99].try_into().unwrap());
        let fee_ratio = u16::from_le_bytes(src[99..101].try_into().unwrap());
        let last_fee_collection_timestamp = u64::from_le_bytes(src[101..109].try_into().unwrap());
        let fee_collection_period = u64::from_le_bytes(src[109..117].try_into().unwrap());
        Ok(Self {
            serum_program_id,
            seed,
            signal_provider,
            status,
            number_of_markets,
            fee_ratio,
            last_fee_collection_timestamp,
            fee_collection_period,
        })
    }

    fn unpack(input: &[u8]) -> Result<Self, ProgramError>
    where
        Self: IsInitialized,
    {
        let value = Self::unpack_unchecked(input)?;
        if value.is_initialized() {
            Ok(value)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    }

    fn unpack_unchecked(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::unpack_from_slice(input)?)
    }

    fn pack(src: Self, dst: &mut [u8]) -> Result<(), ProgramError> {
        if dst.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        src.pack_into_slice(dst);
        Ok(())
    }
}

impl IsInitialized for PoolHeader {
    fn is_initialized(&self) -> bool {
        if let PoolStatus::Uninitialized = self.status {
            return false;
        }
        return true;
    }
}

impl Sealed for PoolAsset {}

impl IsInitialized for PoolAsset {
    fn is_initialized(&self) -> bool {
        self.mint_address != Pubkey::new(&[0u8; 32])
    }
}

impl Pack for PoolAsset {
    const LEN: usize = 32;

    fn pack_into_slice(&self, target: &mut [u8]) {
        let mint_address_bytes = self.mint_address.to_bytes();
        target[0..32].copy_from_slice(&mint_address_bytes);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mint_address = Pubkey::new(&src[..32]);
        Ok(Self { mint_address })
    }
}

pub fn unpack_assets(input: &[u8]) -> Result<Vec<PoolAsset>, ProgramError> {
    let number_of_assets = input.len() / PoolAsset::LEN;
    let mut output: Vec<PoolAsset> = Vec::with_capacity(number_of_assets);
    let mut offset = 0;
    for _ in 0..number_of_assets {
        PoolAsset::unpack(&input[offset..offset + PoolAsset::LEN])
            .and_then(|asset| Ok(output.push(asset)))
            .unwrap_or(());
        offset += PoolAsset::LEN;
    }
    Ok(output)
}

pub fn unpack_unchecked_asset(input: &[u8], index: usize) -> Result<PoolAsset, ProgramError> {
    let offset = index * PoolAsset::LEN;
    input
        .get(offset..offset + PoolAsset::LEN)
        .ok_or(ProgramError::InvalidArgument)
        .and_then(|slice| PoolAsset::unpack_unchecked(slice))
}

pub fn get_asset_slice(target: &mut [u8], index: usize) -> Result<&mut [u8], ProgramError> {
    let offset = index * PoolAsset::LEN;
    target
        .get_mut(offset..offset + PoolAsset::LEN)
        .ok_or(ProgramError::InvalidArgument)
}

pub fn unpack_market(input: &[u8], market_index: u16) -> Pubkey {
    let offset = 32 * (market_index as usize);
    return Pubkey::new(&input[offset..offset + 32]);
}

pub fn pack_markets(target: &mut [u8], markets: &Vec<Pubkey>) -> Result<(), ProgramError> {
    for i in 0..markets.len() {
        target[32 * i..32 * (i + 1)].copy_from_slice(&markets[i].to_bytes());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use super::{pack_markets, unpack_assets, unpack_market, PoolAsset, PoolHeader, PoolStatus};
    use solana_program::{
        program_pack::{IsInitialized, Pack},
        pubkey::Pubkey,
    };

    #[test]
    fn test_state_packing() {
        let header_state = PoolHeader {
            serum_program_id: Pubkey::new_unique(),
            seed: [0u8; 32],
            signal_provider: Pubkey::new_unique(),
            status: PoolStatus::PendingOrder(NonZeroU8::new(39).unwrap()),
            number_of_markets: 234,
            fee_ratio: 15,
            last_fee_collection_timestamp: 1_000_000_000,
            fee_collection_period: 10_000,
        };

        let header_size = PoolHeader::LEN;
        let mut state_array = [0u8; PoolHeader::LEN + 2 * PoolAsset::LEN];
        header_state.pack_into_slice(&mut state_array[..header_size]);

        let pool_asset = PoolAsset {
            mint_address: Pubkey::new_unique(),
        };
        let pool_asset_2 = PoolAsset {
            mint_address: Pubkey::new_unique(),
        };
        pool_asset.pack_into_slice(&mut state_array[header_size..]);
        pool_asset_2.pack_into_slice(&mut state_array[header_size + PoolAsset::LEN..]);

        let unpacked_header = PoolHeader::unpack(&state_array[..PoolHeader::LEN]).unwrap();
        assert_eq!(unpacked_header, header_state);

        let unpacked_pool_assets = unpack_assets(&state_array[PoolHeader::LEN..]).unwrap();
        assert_eq!(unpacked_pool_assets[0], pool_asset);
        assert_eq!(unpacked_pool_assets[1], pool_asset_2);
    }

    #[test]
    fn test_header_packing() {
        let mut header_state = PoolHeader {
            serum_program_id: Pubkey::new_unique(),
            seed: [0u8; 32],
            signal_provider: Pubkey::new_unique(),
            status: PoolStatus::PendingOrder(NonZeroU8::new(39).unwrap()),
            number_of_markets: 234,
            fee_ratio: 15,
            last_fee_collection_timestamp: 1_000_000_000,
            fee_collection_period: 10_000,
        };
        assert_eq!(
            header_state,
            PoolHeader::unpack(&get_packed(&header_state)).unwrap()
        );

        header_state = PoolHeader {
            serum_program_id: Pubkey::new_unique(),
            seed: [0u8; 32],
            signal_provider: Pubkey::new_unique(),
            status: PoolStatus::LockedPendingOrder(NonZeroU8::new(64).unwrap()),
            number_of_markets: 234,
            fee_ratio: 15,
            last_fee_collection_timestamp: 1_000_000_000,
            fee_collection_period: 10_000,
        };
        assert_eq!(
            header_state,
            PoolHeader::unpack(&get_packed(&header_state)).unwrap()
        );

        header_state = PoolHeader {
            serum_program_id: Pubkey::new_unique(),
            seed: [0u8; 32],
            signal_provider: Pubkey::new_unique(),
            status: PoolStatus::Locked,
            number_of_markets: 234,
            fee_ratio: 15,
            last_fee_collection_timestamp: 1_000_000_000,
            fee_collection_period: 10_000,
        };
        assert_eq!(
            header_state,
            PoolHeader::unpack(&get_packed(&header_state)).unwrap()
        );

        header_state = PoolHeader {
            serum_program_id: Pubkey::new_unique(),
            seed: [0u8; 32],
            signal_provider: Pubkey::new_unique(),
            status: PoolStatus::Unlocked,
            number_of_markets: 234,
            fee_ratio: 15,
            last_fee_collection_timestamp: 1_000_000_000,
            fee_collection_period: 10_000,
        };
        assert_eq!(
            header_state,
            PoolHeader::unpack(&get_packed(&header_state)).unwrap()
        );

        header_state = PoolHeader {
            serum_program_id: Pubkey::new_unique(),
            seed: [0u8; 32],
            signal_provider: Pubkey::new_unique(),
            status: PoolStatus::Uninitialized,
            number_of_markets: 234,
            fee_ratio: 15,
            last_fee_collection_timestamp: 1_000_000_000,
            fee_collection_period: 10_000,
        };
        assert!(PoolHeader::unpack(&get_packed(&header_state)).is_err());
    }

    fn get_packed<T: Pack>(obj: &T) -> Vec<u8> {
        let mut output_vec = vec![0u8].repeat(T::LEN);
        obj.pack_into_slice(&mut output_vec);
        output_vec
    }

    #[test]
    fn test_state_init() {
        let pool_asset = PoolAsset::unpack_unchecked(&[0u8; PoolAsset::LEN]).unwrap();
        assert!(!pool_asset.is_initialized());
    }

    #[test]
    fn test_market_packing() {
        let markets = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mut output_array = [0u8; 4 * 32];
        pack_markets(&mut output_array, &markets).unwrap();
        for i in 0..4 {
            assert_eq!(markets[i], unpack_market(&output_array, i as u16));
        }
    }
}
