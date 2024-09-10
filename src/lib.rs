#![allow(dead_code)]

mod bt_memory;
use bt_memory::*;

use asr::future::{next_tick, retry};

use asr::{Address, Address64, Process};

use asr::time::Duration;

asr::async_main!(stable);
static BLOODTHIEF_NAMES: [&str; 2] = [
    "bloodthief_v0.0",      // linux
    "bloodthief_v0.01.exe", // windows
];

async fn main() {
    let p_name = "bloodthief_v0.01.x86_64";
    //let p_name = "fpstest2.x86_64";
    // TODO: Set up some general state and settings.
    asr::set_tick_rate(30.0);
    asr::timer::pause_game_time();

    loop {
        let process = wait_attach_bloodthief().await;
        if let Ok(base_address) = process.get_module_address(p_name) {
            process.until_closes(async {
                // TODO: Initialise some stuff

                let (scene_tree, game_manager_member_array, end_level_screen_ptr) = setup(&process, base_address).await;

                let mut is_in_level = false;
                let mut checkpoint_number: i32 = 0;
                let mut level_is_finished: i32 = 0;
                let mut igt: f64 = 0.0;
                let mut reset_count: i32 = 0;

                loop {
                    next_tick().await;

                    // TODO: Do stuff

                    let Some(current_scene_node) = read_pointer_option(&process, scene_tree + CURRENT_SCENE) else { continue };
                    let Some(current_scene) = &read_node_name_option(&process, current_scene_node) else { continue };

                    let level_was_finished = level_is_finished;
                    let Some(a) = read_int_option(&process, end_level_screen_ptr + LEVEL_END_VISIBLE) else { continue };
                    level_is_finished = a;

                    let old_igt = igt;

                    let Some(a) = read_float_option(&process, game_manager_member_array + GAME_IGT) else { continue };
                    let a = (a - 7.2) / 13.3;
                    igt = a;

                    let old_checkpoint = checkpoint_number;
                    let Some(a) = read_int_option(&process, game_manager_member_array + GAME_CHECKPOINT) else { continue };
                    checkpoint_number = a;

                    let old_reset_count = reset_count;
                    let Some(a) = read_int_option(&process, game_manager_member_array + GAME_RESET_COUNT) else { continue };
                    reset_count = a;
                    
                    // asr::print_message(&igt.to_string());
                    // asr::print_message(&checkpoint_number.to_string());
                    // asr::print_message(&level_is_finished.to_string());
                    // asr::print_message(&current_scene.to_string());

                    let was_in_level = is_in_level;
                    is_in_level = current_scene != "MainScreen";


                    if is_in_level && !was_in_level {
                        // we entered the level
                        asr::timer::reset();
                        asr::timer::start();
                    }


                    if is_in_level {
                        // we are in game
                        asr::timer::set_game_time(Duration::new(igt as i64, ((igt - (igt as i64 as f64)) * 1_000_000_000.0) as i32));

                        if checkpoint_number > old_checkpoint {
                            asr::timer::split();
                            asr::print_message("Should split");
                        }
                        
                        if reset_count > old_reset_count {
                            asr::timer::split();
                            asr::print_message("Should split");
                        }

                        if old_igt > igt {
                            // we hit reset
                            asr::timer::reset();
                            asr::timer::start();
                        }
                    }
                    

                    if level_is_finished == 1 && level_was_finished == 0 {
                        // we just finished
                        asr::timer::split();
                    }


                }
            }).await;
        asr::print_message("Process closed");
        }
    }
}

async fn setup(process: &Process, base_address: Address) -> (Address64, Address64, Address64) {
    loop {
        next_tick().await;

        let scene_tree_ptr = SCENE_TREE_PTR_SIG.wait_scan_process_range(&process, (base_address, 312332123)).await; // the number works idk why and i wont touch it

        let Some(scene_tree)  = read_pointer_option(&process, scene_tree_ptr.value() + SCENE_TREE) else { continue };
        let Some(root_window) = read_pointer_option(&process, scene_tree + ROOT_WINDOW) else { continue };

        let Some(child_count) = read_int_option(&process, root_window + NODE_CHILD_COUNT) else { continue };

        let Some(child_array_ptr) = read_pointer_option(&process, root_window + NODE_CHILD_ARRAY) else { continue };

        let mut game_manager_ptr: Address64 = Address64::new(0);
        let mut end_level_screen_ptr: Address64 = Address64::new(0);

        for i in 0..child_count {
            let Some(child_pointer) = read_pointer_option(&process, child_array_ptr + 0x8 * i) else { break };
            let Some(child_name) = read_node_name_option(&process, child_pointer) else { break };

            // asr::print_message(child_name);
            // asr::print_message(&child_pointer.to_string());

            if child_name == "GameManager" {
                game_manager_ptr = child_pointer;
            }

            if child_name == "EndLevelScreen" {
                end_level_screen_ptr = child_pointer;
            }


            asr::print_message(&child_name);
            asr::print_message(&child_pointer.to_string());
        }

        if game_manager_ptr == Address64::new(0) {
            asr::print_message("Could not find game manager");
            continue;
        }
        if end_level_screen_ptr == Address64::new(0) {
            asr::print_message("Could not find end level screen");
            continue;
        }

        asr::print_message("Found game manager at:");
        asr::print_message(&game_manager_ptr.to_string());
        asr::print_message("Found end level screen at:");
        asr::print_message(&end_level_screen_ptr.to_string());

        let Some(game_manager_script) = read_pointer_option(&process, game_manager_ptr + NODE_SCRIPT) else { continue };
        let Some(game_manager_member_array) = read_pointer_option(&process, game_manager_script + SCRIPT_MEMBER_ARRAY) else { continue };

        return (scene_tree, game_manager_member_array, end_level_screen_ptr);
    }
}

fn read_pointer_option(process: &Process, address: impl Into<Address>) -> Option<Address64> {
    let read_value: Address64 = process.read(address).ok()?;

    return Some(read_value);
}

fn read_int_option(process: &Process, address: impl Into<Address>) -> Option<i32> {
    let read_value: i32 = process.read(address).ok()?;

    return Some(read_value);
}

fn read_float_option(process: &Process, address: impl Into<Address>) -> Option<f64> {
    let read_value: f64 = process.read(address).ok()?;

    return Some(read_value);
}

fn read_string_name_option(process: &Process, start_location: Address64) -> Option<String> {
    let mut output = String::new();
    let mut char_pointer: Address64 = read_pointer_option(&process, start_location + STRING_NAME_START)?;
    // asr::print_message(&char_pointer.to_string());
    // let mut char_pointer: Address64 = read_pointer(&process, char_pointer);
    // asr::print_message(&char_pointer.to_string());

    let mut next_int = read_int_option(process, char_pointer)?;
    while next_int != 0 {
        let next_value: char = char::from_u32(next_int as u32)?;
        //asr::print_message(&next_value.to_string());
        output.push(next_value);
        char_pointer = Address64::from(char_pointer + 0x4);
        next_int = read_int_option(process, char_pointer)?;
    }

    return Some(output);
}

fn read_node_name_option(process: &Process, node_ptr: Address64) -> Option<String> {
    let node_name_ptr: Address64 = read_pointer_option(&process, node_ptr + NODE_NAME)?;
    // asr::print_message("Node name pointer at:");
    // asr::print_message(&node_name_ptr.to_string());
    return read_string_name_option(process, node_name_ptr);
}

async fn wait_attach_bloodthief() -> Process {
    retry(|| {
        attach_bloodthief()
    }).await
}

fn attach_bloodthief() -> Option<Process> {
    BLOODTHIEF_NAMES.into_iter().find_map(Process::attach)
}

