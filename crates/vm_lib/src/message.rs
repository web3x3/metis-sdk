use anyhow::anyhow;
use metis_primitives::{Address, Bytes, U256};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Default Unsigned VM message type which includes all data needed for a state transition
#[derive(PartialEq, Clone, Debug, Hash, Eq)]
#[allow(missing_docs)]
pub struct Message {
    pub version: u64,
    pub from: Address,
    pub to: Address,
    pub sequence: u64,
    pub value: U256,
    pub method_num: u64,
    pub params: Bytes,
    pub gas_limit: u64,
    pub gas_fee_cap: U256,
    pub gas_premium: U256,
}

impl Message {
    /// Does some basic checks on the Message to see if the fields are valid.
    pub fn check(&self) -> anyhow::Result<()> {
        if self.gas_limit == 0 {
            return Err(anyhow!("Message has no gas limit set"));
        }
        // TODO: max block limit?
        if self.gas_limit > i64::MAX as u64 {
            return Err(anyhow!("Message gas exceeds i64 max"));
        }
        Ok(())
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, s: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (
            &self.version,
            &self.to,
            &self.from,
            &self.sequence,
            &self.value,
            &self.gas_limit,
            &self.gas_fee_cap,
            &self.gas_premium,
            &self.method_num,
            &self.params,
        )
            .serialize(s)
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (
            version,
            to,
            from,
            sequence,
            value,
            gas_limit,
            gas_fee_cap,
            gas_premium,
            method_num,
            params,
        ) = Deserialize::deserialize(deserializer)?;
        Ok(Self {
            version,
            from,
            to,
            sequence,
            value,
            method_num,
            params,
            gas_limit,
            gas_fee_cap,
            gas_premium,
        })
    }
}
