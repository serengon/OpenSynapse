#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId {
    pub vid: u16,
    pub pid: u16,
    pub serial: Option<String>,
}

impl DeviceId {
    pub fn matches(&self, other: &DeviceId) -> bool {
        if self.vid != other.vid || self.pid != other.pid {
            return false;
        }
        match (&self.serial, &other.serial) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    Keypad,
    Keyboard,
    Mouse,
    Headset,
    Mousemat,
    Other,
}

impl DeviceKind {
    pub fn from_openrazer(s: &str) -> Self {
        match s {
            "keypad" => Self::Keypad,
            "keyboard" => Self::Keyboard,
            "mouse" => Self::Mouse,
            "headset" => Self::Headset,
            "mousemat" => Self::Mousemat,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub id: DeviceId,
    pub name: String,
    pub kind: DeviceKind,
}
