#![allow(dead_code)]

mod dictionary;
mod auto_splitter_settings;

use bt_memory::{read_pointer, read_int, read_float, read_node_name};
use dictionary::Dictionary;

mod bt_memory;

use asr::future::next_tick;

use asr::{Address, Address64, Process};
use asr::time::Duration;

asr::async_main!(stable);

fn level_number(name: &str) -> isize {
    if name == "JakePractice2" {
        return 1;
    } else if name == "JakePractice3" {
        return 2;
    } else if name == "MysteryCastle2" {
        return 3;
    } else if name == "Dungeon1" {
        return 4;
    } else if name == "Fortress" {
        return 5;
    } else if name == "DoomChapel" {
        return 6;
    }

    return 0;
}

async fn main() {
    // TODO: Set up some general state and settings.
    asr::set_tick_rate(20.0);
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

                let (scene_tree, game_manager_script, end_level_screen_ptr, stats_service_script) = setup(&process, base_address, os).await;
                asr::print_message("Finished setup");

                let Some(igt_ptr) = bt_memory::find_var(&process, os, game_manager_script, "_total_game_seconds_obfuscated") else { return };
                let Some(kill_dict_ptr) = bt_memory::find_var(&process, os, stats_service_script, "_enemies_killed") else { return };



                let mut is_in_level = false;
                let mut level_is_finished: i32 = 0;
                let mut igt: f64 = 0.0;

                let mut total_igt: f64 = 0.0;
                // let mut old_kills = 0;


                loop {
                    next_tick().await;
                    // TODO: Do stuff
                    // kills
                    // let Some(kill_dict_addr) = bt_memory::read_pointer(&process, kill_dict_ptr) else { continue };

                    // let kill_dict = Dictionary::new(kill_dict_addr, 0x18);

                    // let Some(kills) = kill_dict.get_sum(&process) else { continue };
                    // let kills_float: f64 = kills.into();

                    // scene
                    let Some(current_scene_node) = read_pointer(&process, scene_tree + bt_memory::get_current_scene(os)) else { continue };
                    let Some(current_scene) = &read_node_name(&process, current_scene_node, os) else { continue };

                    let level_was_finished = level_is_finished;
                    let Some(a) = read_int(&process, end_level_screen_ptr + bt_memory::get_level_end_visible(os)) else { continue };
                    level_is_finished = a;

                    let old_igt = igt;

                    let Some(a) = read_float(&process, igt_ptr) else { continue };
                    let a = (a - 7.2) / 13.3; //- kills_float * 0.9;
                    igt = a;


                    let was_in_level = is_in_level;
                    is_in_level = current_scene != "MainScreen";




                    // asr::print_message(&current_scene);
                    // asr::print_message(&format!("{}",level_is_finished));

                    if is_in_level && !was_in_level {
                        // we entered the level
                        if level_number(&current_scene) == 1 && level_is_finished != 1 {
                            total_igt = 0.0;
                            asr::timer::reset();
                            asr::timer::start();
                        }
                    }
                    if !is_in_level && was_in_level {
                        //asr::timer::reset();
                    }


                    if is_in_level {
                        // we are in game
                        let actual_time = igt + total_igt;

                        asr::timer::set_game_time(Duration::new(actual_time as i64, ((actual_time - (actual_time as i64 as f64)) * 1_000_000_000.0) as i32));

                        if old_igt > igt {
                            // we hit reset

                            if level_number(&current_scene) == 1 && level_was_finished == 0{
                                asr::timer::reset();
                                asr::timer::start();
                            } 
                            else {
                                total_igt += old_igt;
                            }
                        }
                    }

                    // old_
                    // old_kills = kills;


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
        asr::print_message("Scene tree ptr at");
        let scene_tree_ptr = scene_tree_sig.wait_scan_process_range(&process, (base_address, 312332123)).await; // the number works idk why and i wont touch it
        asr::print_message("Scene tree ptr at");
        asr::print_message(&scene_tree_ptr.to_string());

        let Some(scene_tree_offset) = read_int(&process, scene_tree_ptr + bt_memory::SCENE_TREE_OFFSET) else { continue };
        asr::print_message("Scene tree offset");
        asr::print_message(&scene_tree_offset.to_string());


        let Some(scene_tree)  = read_pointer(&process, scene_tree_ptr + scene_tree_offset + bt_memory::SCENE_TREE) else { continue };
        asr::print_message("Scene tree at");
        asr::print_message(&scene_tree.to_string());

        let Some(root_window) = read_pointer(&process, scene_tree + bt_memory::get_root_window(os)) else { continue };
        asr::print_message("Root window at");
        asr::print_message(&root_window.to_string());

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

        let Some(stats_service_script) = read_pointer(&process, stats_service_ptr + bt_memory::get_node_script(os)) else { continue };

        return (scene_tree, game_manager_script, end_level_screen_ptr, stats_service_script);
    }
}

