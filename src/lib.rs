#![allow(dead_code)]

mod dictionary;
use dictionary::Dictionary;

mod bt_memory;
use bt_memory::*;

use asr::future::next_tick;

use asr::{Address, Address64, Process};

use asr::time::Duration;

asr::async_main!(stable);

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

                let (scene_tree, game_manager_member_array, end_level_screen_ptr, stats_service_member_array) = setup(&process, base_address).await;

                let mut is_in_level = false;
                let mut checkpoint_number: i32 = 0;
                let mut level_is_finished: i32 = 0;
                let mut igt: f64 = 0.0;
                let mut reset_count: i32 = 0;
                let mut sum_secrets: u32 = 0;


                loop {
                    next_tick().await;
                    let Some(dictionary_pointer) = read_pointer(&process, stats_service_member_array + SECRET_STAT) else { return };
                    let secrets_dict: Dictionary = Dictionary::new(dictionary_pointer);

                    // TODO: Do stuff

                    let Some(current_scene_node) = read_pointer(&process, scene_tree + CURRENT_SCENE) else { continue };
                    let Some(current_scene) = &read_node_name(&process, current_scene_node) else { continue };

                    let level_was_finished = level_is_finished;
                    let Some(a) = read_int(&process, end_level_screen_ptr + LEVEL_END_VISIBLE) else { continue };
                    level_is_finished = a;

                    let old_igt = igt;

                    let Some(a) = read_float(&process, game_manager_member_array + GAME_IGT) else { continue };
                    let a = (a - 7.2) / 13.3;
                    igt = a;

                    let old_checkpoint = checkpoint_number;
                    let Some(a) = read_int(&process, game_manager_member_array + GAME_CHECKPOINT) else { continue };
                    checkpoint_number = a;

                    let _old_reset_count = reset_count;
                    let Some(a) = read_int(&process, game_manager_member_array + GAME_RESET_COUNT) else { continue };
                    reset_count = a;
                    
                    let Some(length) = secrets_dict.get_length(&process) else { continue };
                    let old_sum: u32 = sum_secrets;
                    sum_secrets = 0;

                    if length > 0 {
                        let Some(secrets_key_value_pairs) = secrets_dict.get_key_addr_pairs(&process) else { continue };
                        // asr::print_message("AAA");

                        for i in secrets_key_value_pairs {
                            // asr::print_message(&i.0.to_string());
                            // asr::print_message(&i.1.to_string());
                            // asr::print_message("b");
                            let (_key_addr, value_addr) = i;
                            let Some(value) = read_int(&process, value_addr) else { continue };


                            if value == 1 {
                                sum_secrets += 1;
                            }
                        }
                    }

                    // asr::print_message(&sum_enemies.to_string());
                    // asr::print_message(&old_sum.to_string());
                    if sum_secrets == 1 && old_sum != 1 {
                        asr::timer::split();
                        asr::print_message("Split on secret stuff")
                    }

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

async fn setup(process: &Process, base_address: Address) -> (Address64, Address64, Address64, Address64) {
    loop {
        next_tick().await;

        let scene_tree_ptr = SCENE_TREE_PTR_SIG.wait_scan_process_range(&process, (base_address, 312332123)).await; // the number works idk why and i wont touch it

        let Some(scene_tree)  = read_pointer(&process, scene_tree_ptr.value() + SCENE_TREE) else { continue };
        let Some(root_window) = read_pointer(&process, scene_tree + ROOT_WINDOW) else { continue };

        let Some(child_count) = read_int(&process, root_window + NODE_CHILD_COUNT) else { continue };

        let Some(child_array_ptr) = read_pointer(&process, root_window + NODE_CHILD_ARRAY) else { continue };

        let mut game_manager_ptr: Address64 = Address64::new(0);
        let mut end_level_screen_ptr: Address64 = Address64::new(0);
        let mut stats_service_ptr: Address64 = Address64::new(0);

        for i in 0..child_count {
            let Some(child_pointer) = read_pointer(&process, child_array_ptr + 0x8 * i) else { break };
            let Some(child_name) = read_node_name(&process, child_pointer) else { break };

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

        let Some(game_manager_script) = read_pointer(&process, game_manager_ptr + NODE_SCRIPT) else { continue };
        let Some(game_manager_member_array) = read_pointer(&process, game_manager_script + SCRIPT_MEMBER_ARRAY) else { continue };

        let Some(stats_service_script) = read_pointer(&process, stats_service_ptr + NODE_SCRIPT) else { continue };
        let Some(stats_service_member_array) = read_pointer(&process, stats_service_script + SCRIPT_MEMBER_ARRAY) else { continue };

        return (scene_tree, game_manager_member_array, end_level_screen_ptr, stats_service_member_array);
    }
}

