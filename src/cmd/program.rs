//! Command program.
use crate::{
    api::Api,
    metadata::Metadata,
    result::{Error, Result},
};
use structopt::StructOpt;
use subxt::sp_core::H256;

/// Read program state, etc.
#[derive(Clone, Debug, StructOpt)]
pub enum Action {
    /// Read program state.
    State {
        /// Input message for reading program state.
        #[structopt(short, long, default_value = "0x")]
        msg: String,
        /// Block timestamp.
        #[structopt(short, long)]
        timestamp: Option<u64>,
        /// Block height.
        #[structopt(short, long)]
        height: Option<u64>,
    },
}

/// Read program state, etc.
#[derive(Debug, StructOpt)]
pub struct Program {
    /// Program id.
    pid: String,
    #[structopt(subcommand)]
    action: Action,
}

impl Program {
    /// Run command program.
    pub async fn exec(&self, api: Api) -> Result<()> {
        let pid_bytes = hex::decode(&self.pid.trim_start_matches("0x"))?;
        let mut pid = [0; 32];
        pid.copy_from_slice(&pid_bytes);

        match self.action {
            Action::State { .. } => self.state(api, pid.into()).await?,
        }

        Ok(())
    }

    /// Read program state.
    pub async fn state(&self, api: Api, pid: H256) -> Result<()> {
        let Action::State {
            msg,
            timestamp,
            height,
        } = self.action.clone();

        let code = api
            .code_storage(pid.0)
            .await?
            .ok_or_else(|| Error::CodeNotFound(self.pid.clone()))?;

        let pages = api.program_pages(pid).await?;
        let state = Metadata::read(
            &code.code,
            code.static_pages.0 as u64,
            pages,
            hex::decode(msg)?,
            timestamp.unwrap_or(0),
            height.unwrap_or(0),
        )?;

        println!("state: 0x{}", hex::encode(state));

        Ok(())
    }
}
