use crate::dao::call_sign::NewCallSign;
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystem};
use crate::{RepeaterAtlasError, dao};
use diesel_async::AsyncPgConnection;
use tracing::info;

pub async fn create_repeater_system(
    c: &mut AsyncPgConnection,
    repeater: NewRepeaterSystem,
) -> Result<RepeaterSystem, RepeaterAtlasError> {
    let call_sign = repeater.call_sign.clone();
    let new_call_sign = NewCallSign::new_repeater(call_sign.clone());

    dao::call_sign::insert(c, new_call_sign)
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(
                e,
                format!("call_sign kind=repeater value={call_sign}"),
            )
        })?;

    info!("Creating repeater system call sign {call_sign}");

    let system = dao::repeater_system::insert(c, repeater)
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(e, format!("repeater system call_sign={call_sign}"))
        })?;

    Ok(system)
}
