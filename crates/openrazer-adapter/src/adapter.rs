use async_trait::async_trait;
use opensynapse_core::{
    AdapterError, DeviceDiscovery, DeviceId, DeviceKind, DiscoveredDevice, LightingAdapter,
    LightingMode, LightingSpec, Result as CoreResult,
};
use zbus::Connection;

use crate::error::Result;
use crate::proxy::{BatteryProxy, BrightnessProxy, ChromaProxy, DevicesProxy, MiscProxy};

pub struct OpenrazerAdapter {
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

impl OpenrazerAdapter {
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

#[async_trait]
impl DeviceDiscovery for OpenrazerAdapter {
    async fn discover(&self) -> CoreResult<Vec<DiscoveredDevice>> {
        let handles = self.list_devices().await.map_err(adapter_to_core)?;
        let mut out = Vec::with_capacity(handles.len());
        for h in handles {
            let info = h.info().await.map_err(adapter_to_core)?;
            out.push(DiscoveredDevice {
                id: DeviceId {
                    vid: info.vid,
                    pid: info.pid,
                    serial: Some(info.serial),
                },
                name: info.name,
                kind: DeviceKind::from_openrazer(&info.kind),
            });
        }
        Ok(out)
    }
}

impl OpenrazerAdapter {
    async fn resolve(&self, device: &DeviceId) -> CoreResult<DeviceHandle> {
        let handles = self.list_devices().await.map_err(adapter_to_core)?;
        for h in handles {
            let info = h.info().await.map_err(adapter_to_core)?;
            let candidate = DeviceId {
                vid: info.vid,
                pid: info.pid,
                serial: Some(info.serial),
            };
            if device.matches(&candidate) {
                return Ok(h);
            }
        }
        Err(AdapterError::DeviceNotFound(device.clone()))
    }
}

#[async_trait]
impl LightingAdapter for OpenrazerAdapter {
    async fn apply_lighting(&self, device: &DeviceId, spec: &LightingSpec) -> CoreResult<()> {
        let handle = self.resolve(device).await?;
        let chroma = ChromaProxy::builder(&handle.conn)
            .path(handle.path())
            .map_err(zbus_to_core)?
            .build()
            .await
            .map_err(zbus_to_core)?;

        let require_color = || -> CoreResult<opensynapse_core::Color> {
            spec.color.ok_or_else(|| {
                AdapterError::unsupported(format!("mode {:?} requires color", spec.mode))
            })
        };

        match spec.mode {
            LightingMode::None => chroma.set_none().await.map_err(zbus_to_core)?,
            LightingMode::Static => {
                let c = require_color()?;
                chroma
                    .set_static(c.r, c.g, c.b)
                    .await
                    .map_err(zbus_to_core)?;
            }
            LightingMode::Breathing => {
                let c = require_color()?;
                chroma
                    .set_breath_single(c.r, c.g, c.b)
                    .await
                    .map_err(zbus_to_core)?;
            }
            LightingMode::Spectrum => chroma.set_spectrum().await.map_err(zbus_to_core)?,
            LightingMode::Wave => chroma.set_wave(1).await.map_err(zbus_to_core)?,
            LightingMode::Reactive => {
                let c = require_color()?;
                chroma
                    .set_reactive(c.r, c.g, c.b, 2)
                    .await
                    .map_err(zbus_to_core)?;
            }
        }

        if let Some(level) = spec.brightness {
            let bright = BrightnessProxy::builder(&handle.conn)
                .path(handle.path())
                .map_err(zbus_to_core)?
                .build()
                .await
                .map_err(zbus_to_core)?;
            bright
                .set_brightness(f64::from(level))
                .await
                .map_err(zbus_to_core)?;
        }

        Ok(())
    }
}

fn zbus_to_core(e: zbus::Error) -> AdapterError {
    AdapterError::transient(Box::new(e))
}

fn adapter_to_core(e: crate::error::AdapterError) -> AdapterError {
    AdapterError::fatal(Box::new(e))
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
