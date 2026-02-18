use crate::RepeaterAtlasError;
use crate::dao;
use crate::service::repeater_system::{Repeater, load as load_repeater};
use diesel_async::AsyncPgConnection;

pub mod chirp;

#[derive(Debug, Clone, Copy)]
pub struct ExportOptions {
    pub export_rx_tone: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            export_rx_tone: true,
        }
    }
}

pub async fn load_repeaters_for_export(
    c: &mut AsyncPgConnection,
) -> Result<Vec<Repeater>, RepeaterAtlasError> {
    let repeaters = dao::repeater_system::select_with_call_sign(c).await?;
    let mut loaded_repeaters = Vec::with_capacity(repeaters.len());

    for repeater in repeaters {
        loaded_repeaters.push(load_repeater(c, repeater).await?);
    }

    Ok(loaded_repeaters)
}
