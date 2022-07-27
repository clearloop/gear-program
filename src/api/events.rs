//! events api
use crate::{
    api::{
        config::GearConfig,
        generated::api::{
            gear::Event as GearEvent, system::Event as SystemEvent, DispatchError, Event,
        },
        Api,
    },
    Result,
};
use futures_util::StreamExt;
use subxt::{
    codec::Decode,
    events::EventSubscription,
    rpc::Subscription,
    sp_runtime::{generic::Header, traits::BlakeTwo256},
    HasModuleError, ModuleError, RuntimeError, TransactionEvents, TransactionInBlock,
};

/// Generic events
pub type Events<'a> =
    EventSubscription<'a, Subscription<Header<u32, BlakeTwo256>>, GearConfig, Event>;

/// Transaction events
#[allow(unused)]
pub type InBlockEvents = TransactionEvents<GearConfig, Event>;

/// Subxt Error
pub type SubxtError = subxt::Error<DispatchError>;

impl Api {
    /// Subscribe all events
    #[allow(unused)]
    pub async fn events(&self) -> Result<Events<'_>> {
        Ok(self.runtime.events().subscribe().await?)
    }

    /// Capture the dispatch info of any extrinsic and display the weight spent
    pub async fn capture_dispatch_info<'e>(
        &self,
        tx: &TransactionInBlock<'e, GearConfig, DispatchError, Event>,
    ) -> core::result::Result<InBlockEvents, SubxtError> {
        let events = tx.fetch_events().await?;

        // Try to find any errors; return the first one we encounter.
        for (raw, event) in events.iter_raw().zip(events.iter()) {
            let ev = raw?;
            if &ev.pallet == "System" && &ev.variant == "ExtrinsicFailed" {
                Self::capture_weight_info(event?.event);
                let dispatch_error = DispatchError::decode(&mut &*ev.data)?;
                if let Some(error_data) = dispatch_error.module_error_data() {
                    // Error index is utilized as the first byte from the error array.
                    let metadata_lock = self.runtime.client.metadata();
                    let metadata = metadata_lock.read();
                    let details =
                        metadata.error(error_data.pallet_index, error_data.error_index())?;

                    return Err(subxt::Error::Module(ModuleError {
                        pallet: details.pallet().to_string(),
                        error: details.error().to_string(),
                        description: details.description().to_vec(),
                        error_data,
                    }));
                } else {
                    return Err(subxt::Error::Runtime(RuntimeError(dispatch_error)));
                }
            } else if &ev.pallet == "System" && &ev.variant == "ExtrinsicSuccess" {
                Self::capture_weight_info(event?.event);
                break;
            }
        }

        Ok(events)
    }

    /// Parse transaction fee from InBlockEvents
    pub fn capture_weight_info(event: Event) {
        if let Event::System(SystemEvent::ExtrinsicSuccess { dispatch_info })
        | Event::System(SystemEvent::ExtrinsicFailed { dispatch_info, .. }) = event
        {
            println!("\tWeight cost: {:?}", dispatch_info.weight);
        }
    }

    /// Wait for GearEvent.
    pub async fn wait_for(mut events: Events<'_>, wait: fn(GearEvent) -> bool) -> Result<()> {
        while let Some(events) = events.next().await {
            for maybe_event in events?.iter() {
                let event = maybe_event?.event;

                // Exit when extrinsic failed.
                //
                // # Safety
                //
                // The error message will be panicked in another thread.
                if let Event::System(SystemEvent::ExtrinsicFailed { .. }) = event {
                    return Ok(());
                }

                // Exit when success or failure.
                if let Event::Gear(e) = event {
                    println!("\t{e:?}");

                    if wait(e) {
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }
}
