//! command `create`
use super::str_to_arr;
use crate::{api::signer::Signer, result::Result};
use structopt::StructOpt;

/// Deploy program to gear node
#[derive(StructOpt, Debug)]
pub struct Create {
    /// gear program code id
    code_id: String,
    /// gear program salt ( hex encoding )
    #[structopt(default_value = "0x")]
    salt: String,
    /// gear program init payload ( hex encoding )
    #[structopt(default_value = "0x")]
    init_payload: String,
    /// gear program gas limit
    ///
    /// if zero, gear will estimate this automatically
    #[structopt(default_value = "0")]
    gas_limit: u64,
    /// gear program balance
    #[structopt(default_value = "0")]
    value: u128,
}

impl Create {
    /// Exec command submit
    pub async fn exec(&self, signer: Signer) -> Result<()> {
        let code_id = str_to_arr(&self.code_id)?.into();
        let payload = hex::decode(&self.init_payload.trim_start_matches("0x"))?;

        let gas = if self.gas_limit == 0 {
            signer
                .calculate_create_gas(code_id, payload.clone(), self.value, false, None)
                .await?
                .min_limit
        } else {
            self.gas_limit
        };

        // estimate gas
        let gas_limit = signer.cmp_gas_limit(gas).await?;

        // create program
        signer
            .create_program(
                code_id,
                str_to_arr(&self.salt)?.into(),
                payload,
                gas_limit,
                self.value,
            )
            .await?;

        Ok(())
    }
}
