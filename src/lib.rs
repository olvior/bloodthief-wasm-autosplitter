#![allow(dead_code)]

mod dictionary;

use bt_memory::{read_pointer, read_int, read_float, read_node_name};
use dictionary::Dictionary;

mod bt_memory;

use asr::future::next_tick;

use asr::{Address, Address64, Process};

use asr::time::Duration;

asr::async_main!(stable);


async fn main() {
    // TODO: Set up some general state and settings.
    asr::set_tick_rate(30.0);
    asr::timer::pause_game_time();
    // let mut settings = Settings::register();

    loop {

        let Ok(possible_os) = asr::get_os() else { continue };
        let os: &str = &possible_os.to_string();
        asr::print_message(os);

        let p_name = bt_memory::get_p_name(os);
        asr::print_message(p_name);


        let process = bt_memory::wait_attach_bloodthief().await;
        if let Ok(base_address) = process.get_module_address(p_name) {
            process.until_closes(async {
                // TODO: Initialise some stuff

                let (scene_tree, game_manager_member_array, end_level_screen_ptr, stats_service_member_array) = setup(&process, base_address, os).await;

                let mut is_in_level = false;
                let mut checkpoint_number: i32 = 0;
                let mut level_is_finished: i32 = 0;
                let mut igt: f64 = 0.0;
                let mut sum_secrets: u32 = 0;
                let mut sum_keys: u32 = 0;


                loop {
                    next_tick().await;
                    // TODO: Do stuff

                    let Some(current_scene_node) = read_pointer(&process, scene_tree + bt_memory::get_current_scene(os)) else { continue };
                    let Some(current_scene) = &read_node_name(&process, current_scene_node, os) else { continue };

                    let Some(dictionary_pointer) = read_pointer(&process, stats_service_member_array + bt_memory::SECRET_STAT) else { continue };
                    let secrets_dict: Dictionary = Dictionary::new(dictionary_pointer, 0x50);

                    let Some(dictionary_pointer) = read_pointer(&process, stats_service_member_array + bt_memory::KEY_STAT) else { continue };

                    let key_special_offset = match bt_memory::KEY_DICT_WEIRD_START_LEVEL.iter().position(|&x| x == current_scene) {
                        Some(index) => bt_memory::KEY_DICT_WEIRD_START_VALUE[index],
                        None => 0x50, // shouldn't matter anyways but 0x50 works normaly
                    };
                    let keys_dict: Dictionary = Dictionary::new(dictionary_pointer, key_special_offset);

                    let level_was_finished = level_is_finished;
                    let Some(a) = read_int(&process, end_level_screen_ptr + bt_memory::get_level_end_visible(os)) else { continue };
                    level_is_finished = a;

                    let old_igt = igt;

                    let Some(a) = read_float(&process, game_manager_member_array + bt_memory::GAME_IGT) else { continue };
                    let a = (a - 7.2) / 13.3;
                    igt = a;

                    let old_checkpoint = checkpoint_number;
                    let Some(a) = read_int(&process, game_manager_member_array + bt_memory::GAME_CHECKPOINT) else { continue };
                    checkpoint_number = a;

                    let old_sum: u32 = sum_secrets;
                    let Some((sum_good, sum)) = secrets_dict.get_sum(&process) else { continue };
                    sum_secrets = sum as u32;

                    if sum_good {
                        if sum_secrets == 1 && old_sum != 1 {
                            asr::timer::split();
                            asr::print_message("Split on secret stuff")
                        }
                    }

                    let old_sum: u32 = sum_keys;
                    let Some((sum_good, sum)) = keys_dict.get_sum(&process) else { continue };
                    sum_keys = sum as u32;

                    if sum_good {
                        if sum_keys > old_sum {
                            asr::timer::split();
                            asr::print_message("Split on key")
                        }
                    }


                    let was_in_level = is_in_level;
                    is_in_level = current_scene != "MainScreen";


                    if is_in_level && !was_in_level {
                        // we entered the level
                        asr::timer::reset();
                        asr::timer::start();
                    }
                    if !is_in_level && was_in_level {
                        asr::timer::reset();
                    }


                    if is_in_level {
                        // we are in game
                        asr::timer::set_game_time(Duration::new(igt as i64, ((igt - (igt as i64 as f64)) * 1_000_000_000.0) as i32));

                        if checkpoint_number > old_checkpoint {
                            asr::timer::split();
                            asr::print_message("split on checkpoint");
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
                        asr::print_message("Split on finish");
                    }


                }
            }).await;
        asr::print_message("Process closed");
        }
    }
}

async fn setup(process: &Process, base_address: Address, os: &str) -> (Address64, Address64, Address64, Address64) {
    loop {
        next_tick().await;

        let scene_tree_sig = bt_memory::get_scene_tree_sig(os);
        let scene_tree_ptr = scene_tree_sig.wait_scan_process_range(&process, (base_address, 312332123)).await; // the number works idk why and i wont touch it

        let Some(scene_tree)  = read_pointer(&process, scene_tree_ptr.value() + bt_memory::get_scene_tree(os)) else { continue };
        let Some(root_window) = read_pointer(&process, scene_tree + bt_memory::get_root_window(os)) else { continue };

        let Some(child_count) = read_int(&process, root_window + bt_memory::get_node_child_count(os)) else { continue };

        let Some(child_array_ptr) = read_pointer(&process, root_window + bt_memory::get_node_child_array(os)) else { continue };

        let mut game_manager_ptr: Address64 = Address64::new(0);
        let mut end_level_screen_ptr: Address64 = Address64::new(0);
        let mut stats_service_ptr: Address64 = Address64::new(0);

        for i in 0..child_count {
            let Some(child_pointer) = read_pointer(&process, child_array_ptr + 0x8 * i) else { break };
            let Some(child_name)    = read_node_name(&process, child_pointer, os) else { break };

            // asr::print_message(child_name);
            // asr::print_message(&child_pointer.to_string());

            if child_name == "GameManager" {
                game_manager_ptr = child_pointer;
            }

            if child_name == "StatsService" {
                stats_service_ptr = child_pointer;
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
        if stats_service_ptr == Address64::new(0) {
            asr::print_message("Could not find stats service");
            continue;
        }

        asr::print_message("Found game manager at:");
        asr::print_message(&game_manager_ptr.to_string());
        asr::print_message("Found end level screen at:");
        asr::print_message(&end_level_screen_ptr.to_string());

        let Some(game_manager_script) = read_pointer(&process, game_manager_ptr + bt_memory::get_node_script(os)) else { continue };
        let Some(game_manager_member_array) = read_pointer(&process, game_manager_script + bt_memory::get_script_member_array(os)) else { continue };

        let Some(stats_service_script) = read_pointer(&process, stats_service_ptr + bt_memory::get_node_script(os)) else { continue };
        let Some(stats_service_member_array) = read_pointer(&process, stats_service_script + bt_memory::get_script_member_array(os)) else { continue };

        return (scene_tree, game_manager_member_array, end_level_screen_ptr, stats_service_member_array);
    }
}

