use revm::primitives::hardfork::SpecId;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Clone, Copy)]
pub enum SpecName {
    Frontier,
    FrontierToHomesteadAt5,
    Homestead,
    HomesteadToDaoAt5,
    HomesteadToEIP150At5,
    EIP150,
    EIP158, // EIP-161: State trie clearing
    EIP158ToByzantiumAt5,
    Byzantium,
    ByzantiumToConstantinopleAt5, // SKIPPED
    ByzantiumToConstantinopleFixAt5,
    Constantinople, // SKIPPED
    ConstantinopleFix,
    Istanbul,
    Berlin,
    BerlinToLondonAt5,
    London,
    Paris,
    Merge,
    Shanghai,
    Cancun,
    Prague,
    Osaka,
    #[serde(other)]
    Unknown,
}

impl SpecName {
    pub fn to_spec_id(&self) -> SpecId {
        match self {
            Self::Frontier => SpecId::FRONTIER,
            Self::Homestead | Self::FrontierToHomesteadAt5 => SpecId::HOMESTEAD,
            Self::EIP150 | Self::HomesteadToDaoAt5 | Self::HomesteadToEIP150At5 => {
                SpecId::TANGERINE
            }
            Self::EIP158 => SpecId::SPURIOUS_DRAGON,
            Self::Byzantium | Self::EIP158ToByzantiumAt5 => SpecId::BYZANTIUM,
            Self::ConstantinopleFix | Self::ByzantiumToConstantinopleFixAt5 => SpecId::PETERSBURG,
            Self::Istanbul => SpecId::ISTANBUL,
            Self::Berlin => SpecId::BERLIN,
            Self::London | Self::BerlinToLondonAt5 => SpecId::LONDON,
            Self::Paris | Self::Merge => SpecId::MERGE,
            Self::Shanghai => SpecId::SHANGHAI,
            Self::Cancun => SpecId::CANCUN,
            Self::Prague => SpecId::PRAGUE,
            Self::Osaka => SpecId::OSAKA,
            Self::ByzantiumToConstantinopleAt5 | Self::Constantinople => {
                panic!("Overridden with PETERSBURG")
            }
            Self::Unknown => panic!("Unknown spec"),
        }
    }
}

pub fn get_block_spec(timestamp: u64, block_number: u64) -> SpecId {
    if timestamp >= 1710338135 {
        SpecId::CANCUN
    } else if timestamp >= 1681338455 {
        SpecId::SHANGHAI
    } else if block_number >= 15537394 {
        SpecId::MERGE
    } else if block_number >= 12965000 {
        SpecId::LONDON
    } else if block_number >= 12244000 {
        SpecId::BERLIN
    } else if block_number >= 9069000 {
        SpecId::ISTANBUL
    } else if block_number >= 7280000 {
        SpecId::PETERSBURG
    } else if block_number >= 4370000 {
        SpecId::BYZANTIUM
    } else if block_number >= 2675000 {
        SpecId::SPURIOUS_DRAGON
    } else if block_number >= 2463000 {
        SpecId::TANGERINE
    } else if block_number >= 1150000 {
        SpecId::HOMESTEAD
    } else {
        SpecId::FRONTIER
    }
}

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    let mut paths = if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension() == Some("json".as_ref()))
            .map(DirEntry::into_path)
            .collect()
    };
    paths.sort();
    paths
}
