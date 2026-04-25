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
