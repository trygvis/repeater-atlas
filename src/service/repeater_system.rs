use crate::dao::call_sign::NewCallSign;
use crate::dao::contact::Contact;
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystem};
use crate::{MaidenheadLocator, RepeaterAtlasError, dao, service};
use diesel_async::AsyncPgConnection;
use tracing::info;

pub async fn create_repeater_system(
    c: &mut AsyncPgConnection,
    call_sign: impl Into<String> + std::fmt::Display,
    owner: Option<&Contact>,
    address: impl Into<String>,
    maidenhead: Option<String>,
) -> Result<RepeaterSystem, RepeaterAtlasError> {
    let call_sign = call_sign.into();

    let call_sign_row = dao::call_sign::insert(c, NewCallSign::new_repeater(&call_sign))
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(
                e,
                format!("call_sign kind=repeater value={call_sign}"),
            )
        })?;

    let mut repeater = NewRepeaterSystem::new(call_sign_row.value.clone());
    if let Some(owner) = owner {
        repeater = repeater.owner(owner.id);
    }
    let address = address.into();
    if !address.trim().is_empty() {
        repeater.address = Some(address);
    }
    repeater.maidenhead = maidenhead
        .map(MaidenheadLocator::new)
        .transpose()
        .map_err(|e| {
            RepeaterAtlasError::Other(
                Box::new(e),
                format!("invalid maidenhead locator for call_sign={call_sign}"),
            )
        })?;

    let geocoder = service::geocoding::nominatim_geocoder_from_env()?;
    if let Some(enriched) = service::enrich_location::enrich_location(
        geocoder,
        &call_sign,
        repeater.address.as_deref(),
        repeater.maidenhead.as_ref(),
    )
    .await?
    {
        repeater.latitude = Some(enriched.latitude);
        repeater.longitude = Some(enriched.longitude);
        repeater.maidenhead = Some(enriched.maidenhead);
    }

    info!("Creating repeater system call sign {call_sign}");

    let system = dao::repeater_system::insert(c, repeater)
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(e, format!("repeater system call_sign={call_sign}"))
        })?;

    Ok(system)
}
