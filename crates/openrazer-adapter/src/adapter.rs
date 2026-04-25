use zbus::Connection;

use crate::error::Result;
use crate::proxy::{BatteryProxy, DevicesProxy, MiscProxy};

pub struct Adapter {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub serial: String,
    pub kind: String,
    pub vid: u16,
    pub pid: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct BatteryState {
    pub level: f64,
    pub charging: bool,
}

pub struct DeviceHandle {
    conn: Connection,
    serial: String,
}

impl Adapter {
    pub async fn connect() -> Result<Self> {
        let conn = Connection::session().await?;
        Ok(Self { conn })
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceHandle>> {
        let proxy = DevicesProxy::new(&self.conn).await?;
        let serials = proxy.get_devices().await?;
        Ok(serials
            .into_iter()
            .map(|serial| DeviceHandle {
                conn: self.conn.clone(),
                serial,
            })
            .collect())
    }
}

fn is_missing(err: &zbus::Error) -> bool {
    match err {
        zbus::Error::MethodError(name, _, _) => {
            let n = name.as_str();
            n == "org.freedesktop.DBus.Error.UnknownMethod"
                || n == "org.freedesktop.DBus.Error.UnknownInterface"
                || n == "org.freedesktop.DBus.Error.UnknownProperty"
        }
        zbus::Error::FDO(boxed) => matches!(
            **boxed,
            zbus::fdo::Error::UnknownMethod(_)
                | zbus::fdo::Error::UnknownInterface(_)
                | zbus::fdo::Error::UnknownProperty(_)
        ),
        _ => false,
    }
}

impl DeviceHandle {
    pub fn serial(&self) -> &str {
        &self.serial
    }

    fn path(&self) -> String {
        format!("/org/razer/device/{}", self.serial)
    }

    pub async fn info(&self) -> Result<DeviceInfo> {
        let proxy = MiscProxy::builder(&self.conn)
            .path(self.path())?
            .build()
            .await?;
        let name = proxy.get_device_name().await?;
        let serial = proxy.get_serial().await?;
        let kind = proxy.get_device_type().await?;
        let vidpid = proxy.get_vid_pid().await?;
        let (vid, pid) = match vidpid.as_slice() {
            [v, p] => (*v as u16, *p as u16),
            _ => (0, 0),
        };
        Ok(DeviceInfo {
            name,
            serial,
            kind,
            vid,
            pid,
        })
    }

    pub async fn battery(&self) -> Result<Option<BatteryState>> {
        let proxy = BatteryProxy::builder(&self.conn)
            .path(self.path())?
            .build()
            .await?;
        match proxy.get_battery().await {
            Ok(level) => {
                let charging = proxy.is_charging().await.unwrap_or(false);
                Ok(Some(BatteryState { level, charging }))
            }
            Err(e) if is_missing(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
