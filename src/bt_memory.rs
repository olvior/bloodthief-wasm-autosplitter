use asr::signature::Signature;
use asr::{Process, Address, Address64};
use asr::future::retry;

static BLOODTHIEF_NAMES: [&str; 2] = [
    "bloodthief.x86_64",      // linux
    "bloodthief.exe", // windows
];

pub fn get_p_name(os: &str) -> &str {
    if os == "linux" { "bloodthief.x86_64" }
    else             { "bloodthief.exe" }
}

const SCENE_TREE_PTR_SIG: Signature<20>          = Signature::new("48 8b 05 ?? ?? ?? ?? 48 8b b7 ?? ?? ?? ?? 48 89 fb 48 89 d5");
const WINDOWS_SCREEN_TREE_PTR_SIG: Signature<20> = Signature::new("4C 8B 35 ?? ?? ?? ?? 4D 85 F6 74 7E E8 ?? ?? ?? ?? 49 8B CE");

pub fn get_scene_tree_sig(os: &str) -> Signature<20> {
    if os == "linux" { SCENE_TREE_PTR_SIG }
    else             { WINDOWS_SCREEN_TREE_PTR_SIG }
}

pub const SCENE_TREE_OFFSET: u64 = 0x3;

pub const SCENE_TREE: u64 = 0x7;

// pub const ROOT_WINDOW: u64 = 0x2d0;
pub fn get_root_window(os: &str) -> u64 {
    if os == "linux" { 0x2d0 + 0xa0 }
    else             { 0x348 }
}

// pub const NODE_CHILD_COUNT: u64 = 0x190;
pub fn get_node_child_count(os: &str) -> u64 {
    if os == "linux" { 0x190 + 0x10 }
    else             { 0x1b8 }
}
// pub const NODE_CHILD_ARRAY: u64 = 0x198;
pub fn get_node_child_array(os: &str) -> u64 {
    if os == "linux" { 0x198 + 0x10 }
    else             { 0x1c0 }
}

// pub const NODE_SCRIPT: u64 = 0x68;
pub fn get_node_script(os: &str) -> u64 {
    if os == "linux" { 0x68 }
    else             { 0x68 }
}

pub const SCRIPT_REFERENCE: u64 = 0x18;

pub fn get_member_map(os: &str) -> u64 {
    if os == "linux" { 0x230 }
    else             { 0x258 }
}

// pub const SCRIPT_MEMBER_ARRAY: u64 = 0x28;
pub fn get_script_member_array(os: &str) -> u64 {
    if os == "linux" { 0x28 }
    else             { 0x28 }
}

// pub const NODE_NAME: u64 = 0x1f0;
pub fn get_node_name(os: &str) -> u64 {
    if os == "linux" { 0x1f0 + 0x10 }
    else             { 0x218 }
}
pub const STRING_NAME_START: u64 = 0x10;

// pub const CURRENT_SCENE: u64 = 0x3c0;
pub fn get_current_scene(os: &str) -> u64 {
    if os == "linux" { 0x460 }
    else             { 0x438 }
}

// pub const LEVEL_END_VISIBLE: u64 = 0x41c;
pub fn get_level_end_visible(os: &str) -> u64 {
    if os == "linux" { 0x42c }
    else             { 0x444 }
}



// i think these variables are the same accross os
pub const GAME_IGT: u64 = 0xe0;
pub const GAME_RESET_COUNT: u64 = 0x2f0;
pub const GAME_CHECKPOINT: u64 = 0x248;

pub const SECRET_STAT: u64 = 0x50;
pub const KEY_STAT: u64 = 0x68;


pub const KEY_DICT_WEIRD_START_LEVEL: [&str; 3] = ["MysteryCastle2", "Dungeon1", "Fortress"]; // Levels 3, 4, and 5
pub const KEY_DICT_WEIRD_START_VALUE: [u32; 3] = [0xa0, 0x30, 0x88];

pub fn read_pointer(process: &Process, address: impl Into<Address>) -> Option<Address64> {
    let read_value: Address64 = process.read(address).ok()?;

    return Some(read_value);
}

pub fn read_int(process: &Process, address: impl Into<Address>) -> Option<i32> {
    let read_value: i32 = process.read(address).ok()?;

    return Some(read_value);
}

pub fn read_float(process: &Process, address: impl Into<Address>) -> Option<f64> {
    let read_value: f64 = process.read(address).ok()?;

    return Some(read_value);
}

pub fn read_string_name(process: &Process, start_location: Address64) -> Option<String> {
    let mut output = String::new();
    let mut char_pointer: Address64 = read_pointer(&process, start_location + STRING_NAME_START)?;
    // asr::print_message(&char_pointer.to_string());
    // let mut char_pointer: Address64 = read_pointer(&process, char_pointer);
    // asr::print_message(&char_pointer.to_string());

    let mut next_int = read_int(process, char_pointer)?;
    while next_int != 0 {
        let next_value: char = char::from_u32(next_int as u32)?;
        //asr::print_message(&next_value.to_string());
        output.push(next_value);
        char_pointer = Address64::from(char_pointer + 0x4);
        next_int = read_int(process, char_pointer)?;
    }

    return Some(output);
}

pub fn read_node_name(process: &Process, node_ptr: Address64, os: &str) -> Option<String> {
    let node_name_ptr: Address64 = read_pointer(&process, node_ptr + get_node_name(os))?;
    // asr::print_message("Node name pointer at:");
    // asr::print_message(&node_name_ptr.to_string());
    return read_string_name(process, node_name_ptr);
}

pub async fn wait_attach_bloodthief() -> Process {
    retry(|| {
        attach_bloodthief()
    }).await
}

fn attach_bloodthief() -> Option<Process> {
    BLOODTHIEF_NAMES.into_iter().find_map(Process::attach)
}

pub fn find_var(process: &Process, os: &str, script: Address64, variable: &str) -> Option<Address64> {
    let member_array = read_pointer(process, script + get_script_member_array(os))?;
    let script_reference = read_pointer(process, script + SCRIPT_REFERENCE)?;
    let script_map = read_pointer(process, script_reference + get_member_map(os))?;

    // asr::print_message(&script_map.to_string());
    // asr::print_message(&script.to_string());
    // asr::print_message("found map and array");

    let mut current_member = script_map;

    while current_member != Into::into(0) {
        let name_ptr = read_pointer(process, current_member + 0x10)?;
        let name = read_string_name(process, name_ptr)?;

        // asr::print_message(&name.to_string());

        if name == variable {
            // we found it so we leave
            break;
        }

        current_member = read_pointer(process, current_member)?;
    }

    let index = read_int(process, current_member + 0x18)?;


    let addr: Address64 = member_array + index * 0x18 + 0x8;
    // asr::print_message(&member_array.to_string());
    // asr::print_message(&addr.to_string());

    Some(addr)
}


