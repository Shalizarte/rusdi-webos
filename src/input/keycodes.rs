//! Mapeamento keycodes webOS → ações Rusdi.

use std::collections::HashMap;

pub mod linux_keys {
    pub const KEY_UP: u16 = 103;
    pub const KEY_DOWN: u16 = 108;
    pub const KEY_LEFT: u16 = 105;
    pub const KEY_RIGHT: u16 = 106;
    pub const KEY_OK: u16 = 28;
    pub const KEY_BACK: u16 = 461;
    pub const KEY_HOME: u16 = 102;
    pub const KEY_MENU: u16 = 139;
    pub const KEY_PLAY: u16 = 164;
    pub const KEY_PAUSE: u16 = 119;
    pub const KEY_STOP: u16 = 128;
    pub const KEY_NEXT: u16 = 163;
    pub const KEY_PREVIOUS: u16 = 165;
    pub const KEY_REWIND: u16 = 168;
    pub const KEY_FASTFORWARD: u16 = 167;
    pub const KEY_PLAYPAUSE: u16 = 164;
    pub const KEY_VOLUMEUP: u16 = 115;
    pub const KEY_VOLUMEDOWN: u16 = 114;
    pub const KEY_MUTE: u16 = 113;
    pub const KEY_RED: u16 = 393;
    pub const KEY_GREEN: u16 = 394;
    pub const KEY_YELLOW: u16 = 395;
    pub const KEY_BLUE: u16 = 396;
    pub const KEY_0: u16 = 11; pub const KEY_1: u16 = 2;
    pub const KEY_2: u16 = 3; pub const KEY_3: u16 = 4;
    pub const KEY_4: u16 = 5; pub const KEY_5: u16 = 6;
    pub const KEY_6: u16 = 7; pub const KEY_7: u16 = 8;
    pub const KEY_8: u16 = 9; pub const KEY_9: u16 = 10;
    pub const KEY_INFO: u16 = 138;
    pub const KEY_EPG: u16 = 166;
    pub const KEY_SUBTITLE: u16 = 171;
    pub const KEY_AUDIO: u16 = 172;
    pub const KEY_SEARCH: u16 = 217;
    pub const KEY_EXIT: u16 = 174;
    pub const KEY_POWER: u16 = 116;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RusdiAction {
    Up, Down, Left, Right, Select, Back, Home, ContextMenu,
    Play, Pause, Stop, Next, Previous, FastForward, Rewind, PlayPause,
    VolumeUp, VolumeDown, Mute,
    Red, Green, Yellow, Blue,
    Number(u8),
    Info, Epg, Subtitle, Audio, Search, Exit, Power,
    Unknown(u16),
}

pub fn build_keymap() -> HashMap<u16, RusdiAction> {
    use linux_keys::*;
    use RusdiAction::*;
    let mut map = HashMap::new();
    map.insert(KEY_UP, Up); map.insert(KEY_DOWN, Down);
    map.insert(KEY_LEFT, Left); map.insert(KEY_RIGHT, Right);
    map.insert(KEY_OK, Select); map.insert(KEY_BACK, Back);
    map.insert(KEY_HOME, Home); map.insert(KEY_MENU, ContextMenu);
    map.insert(KEY_PLAY, Play); map.insert(KEY_PAUSE, Pause);
    map.insert(KEY_STOP, Stop); map.insert(KEY_NEXT, Next);
    map.insert(KEY_PREVIOUS, Previous); map.insert(KEY_FASTFORWARD, FastForward);
    map.insert(KEY_REWIND, Rewind); map.insert(KEY_PLAYPAUSE, PlayPause);
    map.insert(KEY_VOLUMEUP, VolumeUp); map.insert(KEY_VOLUMEDOWN, VolumeDown);
    map.insert(KEY_MUTE, Mute);
    map.insert(KEY_RED, Red); map.insert(KEY_GREEN, Green);
    map.insert(KEY_YELLOW, Yellow); map.insert(KEY_BLUE, Blue);
    for (k, v) in [(KEY_0,0),(KEY_1,1),(KEY_2,2),(KEY_3,3),(KEY_4,4),
                   (KEY_5,5),(KEY_6,6),(KEY_7,7),(KEY_8,8),(KEY_9,9)] {
        map.insert(k, Number(v));
    }
    map.insert(KEY_INFO, Info); map.insert(KEY_EPG, Epg);
    map.insert(KEY_SUBTITLE, Subtitle); map.insert(KEY_AUDIO, Audio);
    map.insert(KEY_SEARCH, Search); map.insert(KEY_EXIT, Exit);
    map.insert(KEY_POWER, Power);
    map
}

pub fn keycode_to_action(code: u16) -> RusdiAction {
    build_keymap().get(&code).copied().unwrap_or(RusdiAction::Unknown(code))
}

pub fn action_to_string(action: RusdiAction) -> &'static str {
    use RusdiAction::*;
    match action {
        Up => "Up", Down => "Down", Left => "Left", Right => "Right",
        Select => "Select", Back => "Back", Home => "Home", ContextMenu => "ContextMenu",
        Play => "Play", Pause => "Pause", Stop => "Stop", Next => "Next",
        Previous => "Previous", FastForward => "FastForward", Rewind => "Rewind",
        PlayPause => "PlayPause", VolumeUp => "VolumeUp", VolumeDown => "VolumeDown",
        Mute => "Mute", Red => "Red", Green => "Green", Yellow => "Yellow", Blue => "Blue",
        Number(n) => match n { 0=>"Number0",1=>"Number1",2=>"Number2",3=>"Number3",
            4=>"Number4",5=>"Number5",6=>"Number6",7=>"Number7",8=>"Number8",9=>"Number9",_=>"Unknown" },
        Info => "Info", Epg => "EPG", Subtitle => "Subtitle", Audio => "Audio",
        Search => "Search", Exit => "Exit", Power => "Power",
        Unknown(_) => "Unknown",
    }
}
