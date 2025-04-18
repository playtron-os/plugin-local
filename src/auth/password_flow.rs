use super::user::UserSignals;
use crate::local::service::LocalService;
use zbus::fdo;
use zbus::object_server::SignalEmitter;
use zbus_macros::interface;

pub struct PasswordFlow {
    pub service: LocalService,
}

impl PasswordFlow {
    pub fn new(service: LocalService) -> Self {
        PasswordFlow { service }
    }
}

#[interface(name = "one.playtron.auth.PasswordFlow")]
impl PasswordFlow {
    /// Log in to the provider with the given username and password
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.SteamBus \
    ///   /one/playtron/SteamBus/SteamClient0 \
    ///   one.playtron.auth.PasswordFlow \
    ///   Login "ss" "PlaytronTest1" "mysecretpassword"
    async fn login(
        &mut self,
        name: String,
        password: String,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        log::info!("Logging in as {:?}", name);
        match self.service.login(name.clone(), password).await {
            Ok(()) => Ok(()),
            Err(e) => {
                log::error!("Login failed: {:?}", e);
                emitter.auth_error(e.to_string()).await?;
                Err(fdo::Error::Failed(e.to_string()))
            }
        }
    }
}
