use asr::signature::Signature;
use asr::{Process, Address, Address64};
use asr::future::retry;

static BLOODTHIEF_NAMES: [&str; 2] = [
    "bloodthief_v0.0",      // linux
    "bloodthief_v0.01.exe", // windows
];

pub static SCENE_TREE_PTR_SIG: Signature<20> = Signature::new("48 8b 05 ?? ?? ?? ?? 48 8b b7 ?? ?? ?? ?? 48 89 fb 48 89 d5");

pub const SCENE_TREE: u64 = 0x3 + 0x3fcb72a + 0x4;
pub const ROOT_WINDOW: u64 = 0x2d0;

pub const NODE_CHILD_COUNT: u64 = 0x190;
pub const NODE_CHILD_ARRAY: u64 = 0x198;

pub const NODE_SCRIPT: u64 = 0x68;
pub const SCRIPT_MEMBER_ARRAY: u64 = 0x28;

pub const NODE_NAME: u64 = 0x1f0;
pub const STRING_NAME_START: u64 = 0x10;

pub const CURRENT_SCENE: u64 = 0x3c0;

pub const LEVEL_END_VISIBLE: u64 = 0x41c;

pub const GAME_IGT: u64 = 0xe0;
pub const GAME_RESET_COUNT: u64 = 0x2f0;
pub const GAME_CHECKPOINT: u64 = 0x230;

// Dictionary i think:
// once you index it, 0x18, then 0x50. You have first item.
// 0x0 for next
// 0x30 for value
// 0x18 for key
//
// just dict -> 0x3c = length of dict
//
// TODO: Some sort of dict parser thing
// Could be good for keys, secrets
// maybe even enemies in the future
//
// other TODO: HK-like split checker, so it doesnt just split randomly
// Good once there are many possibilites for splits
//
// also i just wanted to write:
// place of variable =
// 8 + 24 * the index



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

pub fn read_node_name(process: &Process, node_ptr: Address64) -> Option<String> {
    let node_name_ptr: Address64 = read_pointer(&process, node_ptr + NODE_NAME)?;
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

