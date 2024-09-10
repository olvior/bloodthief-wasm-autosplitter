use asr::signature::Signature;

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

