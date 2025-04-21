use crate::constants::CLIENT_PATH;
use crate::local::service::LocalService;
use crate::plugin::dbus::CONNECTION;
use crate::types::app::ProviderStatus;
use crate::types::results::EmptyResult;
use zbus::object_server::SignalEmitter;
use zbus::{fdo, interface};

pub struct User {
    pub service: LocalService,
}

impl User {
    pub async fn emit_new_user_state_update() -> EmptyResult {
        let con = CONNECTION.lock().await;
        if let Some(connection) = con.as_ref() {
            let user_iface = connection
                .object_server()
                .interface::<_, User>(CLIENT_PATH)
                .await?;
            let interface = user_iface.get().await;
            interface
                .avatar_changed(user_iface.signal_emitter())
                .await?;
            interface
                .username_changed(user_iface.signal_emitter())
                .await?;
            interface
                .identifier_changed(user_iface.signal_emitter())
                .await?;
            interface
                .status_changed(user_iface.signal_emitter())
                .await?;
        }
        Ok(())
    }
}

#[interface(name = "one.playtron.auth.User")]
impl User {
    /// An url to the avatar image. Empty if no avatar is available.
    ///
    /// busctl --user get-property one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.auth.User \
    ///   Avatar
    #[zbus(property)]
    async fn avatar(&self) -> &str {
        ""
    }
    #[zbus(property)]
    async fn set_avatar(&mut self, _avatar: String) {
        log::info!("Setting avatar to {_avatar}");
    }

    /// Internal identifier for given user. Value provided here shouldn’t be mutable.
    /// E.g usernames that can be changed shouldn’t be used here.
    /// Empty when no user is authenticated
    ///
    /// # Example
    ///
    /// busctl --user get-property one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.auth.User \
    ///   Identifier
    #[zbus(property)]
    async fn identifier(&self) -> String {
        match self.service.get_account().await {
            Some(account) => account.clone(),
            None => "".to_string(),
        }
    }
    #[zbus(property)]
    async fn set_identifier(&mut self, identifier: String) {
        log::info!("Setting identifier to {identifier}");
    }

    /// The status of the provider authentication
    ///
    /// # Example
    ///
    /// busctl --user get-property one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.auth.User \
    ///   Status
    #[zbus(property)]
    async fn status(&self) -> i32 {
        match self.service.get_account().await {
            Some(account) => match account.is_empty() {
                true => ProviderStatus::Unauthorized as i32,
                false => ProviderStatus::Authorized as i32,
            },
            None => ProviderStatus::Unauthorized as i32,
        }
    }
    #[zbus(property)]
    async fn set_status(&mut self, status: String) {
        log::info!("Setting status to {status}");
    }

    /// The displayname/username of the currently authenticated user. This will be empty
    /// if none are authenticated.
    ///
    /// # Example
    ///
    /// busctl --user get-property one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.auth.User \
    ///   Username
    #[zbus(property)]
    async fn username(&self) -> String {
        match self.service.get_account().await {
            Some(account) => account.clone(),
            None => "".to_string(),
        }
    }

    /// Emitted when authentication fails or the user is not longer authenticated.
    #[zbus(signal)]
    async fn auth_error(emitter: &SignalEmitter<'_>, value: String) -> zbus::Result<()>;

    /// Switch session to chosen user. Called after initialization of the
    /// plugin is done and session is available. Also used for account
    /// switching. Returns whether it was possible to re-use that session.
    ///
    /// # Example
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.auth.User \
    ///   ChangeUser "s" "PlaytronTest1"
    fn change_user(&self, user_id: String) -> fdo::Result<bool> {
        log::info!("Changing user to {}", user_id);
        Ok(true)
    }

    /// Logs specified user out of the provider.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.auth.User \
    ///   Logout "s" "PlaytronTest1"
    async fn logout(
        &mut self,
        user_id: String,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        log::info!("Logging out {}", user_id);
        match self.service.logout().await {
            Ok(_) => {
                self.status_changed(&emitter).await?;
                self.username_changed(&emitter).await?;
                self.identifier_changed(&emitter).await?;
                Ok(())
            }
            Err(e) => Err(fdo::Error::Failed(e.to_string())),
        }
    }
}
