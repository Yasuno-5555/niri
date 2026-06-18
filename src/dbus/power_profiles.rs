use futures_util::StreamExt;
use zbus::names::InterfaceName;
use zbus::{fdo, zvariant};

pub enum PowerProfilesToNiri {
    ActiveProfileChanged(String),
}

pub fn start(
    to_niri: calloop::channel::Sender<PowerProfilesToNiri>,
) -> anyhow::Result<zbus::blocking::Connection> {
    let conn = zbus::blocking::Connection::system()?;

    let async_conn = conn.inner().clone();
    let future = async move {
        let proxy = fdo::PropertiesProxy::new(
            &async_conn,
            "net.hadess.PowerProfiles",
            "/net/hadess/PowerProfiles",
        )
        .await;
        let proxy = match proxy {
            Ok(x) => x,
            Err(err) => {
                warn!("error creating PropertiesProxy for PowerProfiles: {err:?}");
                return;
            }
        };

        let mut props_changed = match proxy.receive_properties_changed().await {
            Ok(x) => x,
            Err(err) => {
                warn!("error subscribing to PowerProfiles PropertiesChanged: {err:?}");
                return;
            }
        };

        let props = proxy
            .get_all(InterfaceName::try_from("net.hadess.PowerProfiles").unwrap())
            .await;
        let mut props = match props {
            Ok(x) => x,
            Err(err) => {
                warn!("error receiving initial PowerProfiles properties: {err:?}");
                return;
            }
        };

        let mut active_profile = props
            .remove("ActiveProfile")
            .and_then(|value| String::try_from(value).ok())
            .unwrap_or_default();

        if let Err(err) = to_niri.send(PowerProfilesToNiri::ActiveProfileChanged(
            active_profile.clone(),
        )) {
            warn!("error sending initial power profile to niri: {err:?}");
            return;
        };

        while let Some(signal) = props_changed.next().await {
            let args = match signal.args() {
                Ok(args) => args,
                Err(err) => {
                    warn!("error parsing PowerProfiles PropertiesChanged args: {err:?}");
                    return;
                }
            };

            let mut changed = false;
            for (name, value) in args.changed_properties() {
                trace!("power profile property: {name} => {value:?}");
                if *name != "ActiveProfile" {
                    continue;
                }

                let value = zvariant::Str::try_from(value).unwrap_or_default();
                let value = value.as_str();
                if active_profile != value {
                    active_profile = value.to_string();
                    changed = true;
                }
            }

            if !changed {
                continue;
            }

            if let Err(err) = to_niri.send(PowerProfilesToNiri::ActiveProfileChanged(
                active_profile.clone(),
            )) {
                warn!("error sending power profile to niri: {err:?}");
                return;
            }
        }
    };

    let task = conn
        .inner()
        .executor()
        .spawn(future, "monitor power profile changes");
    task.detach();

    Ok(conn)
}
