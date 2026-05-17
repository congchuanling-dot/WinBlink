use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};

pub struct HotkeyManager {
    _manager: GlobalHotKeyManager,
    hotkey_id: u32,
}

impl HotkeyManager {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()?;
        let hotkey = HotKey::new(Some(Modifiers::ALT), Code::Space);
        let hotkey_id = hotkey.id();
        manager.register(hotkey)?;

        Ok(Self {
            _manager: manager,
            hotkey_id,
        })
    }

    pub fn poll(&self) -> bool {
        let receiver = GlobalHotKeyEvent::receiver();
        let mut pressed = false;
        while let Ok(event) = receiver.try_recv() {
            if event.id == self.hotkey_id && event.state == HotKeyState::Pressed {
                pressed = true;
            }
        }
        pressed
    }
}
