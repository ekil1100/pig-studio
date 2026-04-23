use crate::{
    event_bus::{ApplicationEvent, EventBusPort},
    ports::{RuntimeSettings, SettingsStorePort},
};
use chrono::{DateTime, Utc};
use shared_kernel::AppResult;

pub fn execute<S, B>(
    store: &S,
    bus: &B,
    mut settings: RuntimeSettings,
    now: DateTime<Utc>,
) -> AppResult<RuntimeSettings>
where
    S: SettingsStorePort,
    B: EventBusPort,
{
    settings.last_checked_at = Some(now);
    store.save(&settings)?;
    bus.publish(ApplicationEvent::RuntimeSettingsUpdated { checked_at: now });
    Ok(settings)
}
