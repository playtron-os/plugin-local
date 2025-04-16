use zbus::fdo;
use zbus_macros::interface;

pub struct SubscriptionProvider {
    name: String,
    management_url: String,
}

#[interface(name = "one.playtron.plugin.SubscriptionProvider")]
impl SubscriptionProvider {
    /// Returns a list of applications that were added to the
    /// library only thanks to the subscription.
    ///
    fn get_apps(&mut self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }

    /// Name of a subscription
    #[zbus(property)]
    async fn name(&self) -> fdo::Result<String> {
        Ok(self.name.clone())
    }
    #[zbus(property)]
    async fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// URL to a website where management of
    /// a subscription happens.
    #[zbus(property)]
    async fn management_url(&self) -> fdo::Result<String> {
        Ok(self.management_url.clone())
    }
    #[zbus(property)]
    async fn set_management_url(&mut self, management_url: String) {
        self.management_url = management_url;
    }
}
