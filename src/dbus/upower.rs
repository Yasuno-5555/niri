use futures_util::StreamExt;
use zbus::fdo;
use zbus::names::InterfaceName;

#[derive(Debug, Clone)]
pub struct BatteryState {
    pub on_battery: bool,
    pub low_battery: bool,
}

pub enum UPowerToNiri {
    BatteryStateChanged(BatteryState),
}

pub fn start(
    to_niri: calloop::channel::Sender<UPowerToNiri>,
) -> anyhow::Result<zbus::blocking::Connection> {
    let conn = zbus::blocking::Connection::system()?;

    let async_conn = conn.inner().clone();
    let future = async move {
        let proxy = fdo::PropertiesProxy::new(
            &async_conn,
            "org.freedesktop.UPower",
            "/org/freedesktop/UPower",
        )
        .await;
        let proxy = match proxy {
            Ok(x) => x,
            Err(err) => {
                warn!("error creating PropertiesProxy for UPower: {err:?}");
                return;
            }
        };

        let mut props_changed = match proxy.receive_properties_changed().await {
            Ok(x) => x,
            Err(err) => {
                warn!("error subscribing to UPower PropertiesChanged: {err:?}");
                return;
            }
        };

        let props = proxy
            .get_all(InterfaceName::try_from("org.freedesktop.UPower").unwrap())
            .await;
        let mut props = match props {
            Ok(x) => x,
            Err(err) => {
                warn!("error receiving initial UPower properties: {err:?}");
                return;
            }
        };

        let mut state = BatteryState {
            on_battery: props
                .remove("OnBattery")
                .and_then(|value| bool::try_from(value).ok())
                .unwrap_or(false),
            low_battery: props
                .remove("WarningLevel")
                .and_then(|value| u32::try_from(value).ok())
                .is_some_and(|level| level >= 3),
        };

        if let Err(err) = to_niri.send(UPowerToNiri::BatteryStateChanged(state.clone())) {
            warn!("error sending initial UPower state to niri: {err:?}");
            return;
        }

        while let Some(signal) = props_changed.next().await {
            let args = match signal.args() {
                Ok(args) => args,
                Err(err) => {
                    warn!("error parsing UPower PropertiesChanged args: {err:?}");
                    return;
                }
            };

            let mut changed = false;
            for (name, value) in args.changed_properties() {
                trace!("upower property: {name} => {value:?}");
                match *name {
                    "OnBattery" => {
                        let new_value = bool::try_from(value).unwrap_or(state.on_battery);
                        if new_value != state.on_battery {
                            state.on_battery = new_value;
                            changed = true;
                        }
                    }
                    "WarningLevel" => {
                        let new_level = u32::try_from(value).unwrap_or(0);
                        let new_low_battery = new_level >= 3;
                        if new_low_battery != state.low_battery {
                            state.low_battery = new_low_battery;
                            changed = true;
                        }
                    }
                    _ => (),
                }
            }

            if !changed {
                continue;
            }

            if let Err(err) = to_niri.send(UPowerToNiri::BatteryStateChanged(state.clone())) {
                warn!("error sending UPower state to niri: {err:?}");
                return;
            }
        }
    };

    let task = conn
        .inner()
        .executor()
        .spawn(future, "monitor upower property changes");
    task.detach();

    Ok(conn)
}
