use zbus::proxy;

#[proxy(
    interface = "razer.devices",
    default_service = "org.razer",
    default_path = "/org/razer"
)]
pub trait Devices {
    #[zbus(name = "getDevices")]
    fn get_devices(&self) -> zbus::Result<Vec<String>>;
}

#[proxy(interface = "razer.device.misc", default_service = "org.razer")]
pub trait Misc {
    #[zbus(name = "getSerial")]
    fn get_serial(&self) -> zbus::Result<String>;

    #[zbus(name = "getDeviceName")]
    fn get_device_name(&self) -> zbus::Result<String>;

    #[zbus(name = "getDeviceType")]
    fn get_device_type(&self) -> zbus::Result<String>;

    #[zbus(name = "getVidPid")]
    fn get_vid_pid(&self) -> zbus::Result<Vec<i32>>;
}

#[proxy(interface = "razer.device.power", default_service = "org.razer")]
pub trait Battery {
    #[zbus(name = "getBattery")]
    fn get_battery(&self) -> zbus::Result<f64>;

    #[zbus(name = "isCharging")]
    fn is_charging(&self) -> zbus::Result<bool>;
}

#[proxy(
    interface = "razer.device.lighting.chroma",
    default_service = "org.razer"
)]
pub trait Chroma {
    #[zbus(name = "setStatic")]
    fn set_static(&self, r: u8, g: u8, b: u8) -> zbus::Result<()>;

    #[zbus(name = "setBreathSingle")]
    fn set_breath_single(&self, r: u8, g: u8, b: u8) -> zbus::Result<()>;

    #[zbus(name = "setSpectrum")]
    fn set_spectrum(&self) -> zbus::Result<()>;

    #[zbus(name = "setWave")]
    fn set_wave(&self, direction: i32) -> zbus::Result<()>;

    #[zbus(name = "setReactive")]
    fn set_reactive(&self, r: u8, g: u8, b: u8, time: u8) -> zbus::Result<()>;

    #[zbus(name = "setNone")]
    fn set_none(&self) -> zbus::Result<()>;
}

#[proxy(
    interface = "razer.device.lighting.brightness",
    default_service = "org.razer"
)]
pub trait Brightness {
    #[zbus(name = "setBrightness")]
    fn set_brightness(&self, value: f64) -> zbus::Result<()>;
}
