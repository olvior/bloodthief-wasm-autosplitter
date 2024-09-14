use crate::bt_memory;
use asr::{Address64, Process};


/// An element of a godot dictionary
pub struct Element {
    pub address: Address64,
}

impl Element {
    pub fn next(&self, process: &Process) -> Option<Element> {
        let next_address = bt_memory::read_pointer(process, self.address + element_offsets::NEXT)?;
        let element = Element { address: next_address };

        return Some(element);
    }

    pub fn key_address(&self) -> Address64 {
        return self.address + element_offsets::KEY;
    }
    
    pub fn value_address(&self) -> Address64 {
        return self.address + element_offsets::VALUE;
    }
}

/// A struct used to navigate a godot Dictionary
pub struct Dictionary {
    pub address: Address64,
    weird_offset: u32,
}

impl Dictionary {
    pub fn new(address: Address64, weird_offset: u32) -> Dictionary {
        Dictionary {
            address,
            weird_offset,
        }
    }

    pub fn get_first_element(&self, process: &Process) -> Option<Element> {
        let first_address = bt_memory::read_pointer(process, self.address + offsets::FIRST)?;
        let first_address = bt_memory::read_pointer(process, first_address + self.weird_offset)?;

        let first_element = Element { address: first_address };

        return Some(first_element);
    }

    pub fn get_length(&self, process: &Process) -> Option<i32> {
        let len = bt_memory::read_int(process, self.address + offsets::SIZE)?;

        return Some(len);
    }

    // Navigates through all of the elements of the dictionary
    // to return the addresses of the keys and values
    pub fn get_key_addr_pairs(&self, process: &Process) -> Option<Vec<(Address64, Address64)>> {
        let mut values: Vec<(Address64, Address64)> = Vec::new();

        let length = self.get_length(process)?;

        if length == 0 {
            return None;
        }

        let mut current_element = self.get_first_element(process)?;

        for _i in 0..length {
            if current_element.address == Address64::new(0) { break }

            values.push((current_element.key_address(), current_element.value_address()));

            current_element = current_element.next(process)?;
        }


        return Some(values);
    }

    // Gets the sum of all the values
    // The first boolean return false if the length of The
    // dictionary is 0
    pub fn get_sum(&self, process: &Process) -> Option<(bool, i32)> {
        let length = self.get_length(process)?;
        if !(length > 0) {
            return Some((false, 0));
        }

        let key_value_pairs = self.get_key_addr_pairs(process)?;

        let mut sum = 0;
        for (_key_addr, value_addr) in key_value_pairs {
            let current_value = bt_memory::read_int(process, value_addr)?;

            sum += current_value;
        }
        

    return Some((true, sum));
    }
}

mod offsets {
    pub const SIZE: u32 = 0x3c;
    pub const FIRST: u32 = 0x18;
}

mod element_offsets {
    pub const NEXT: u32 = 0x0;
    pub const KEY: u32 = 0x18;
    pub const VALUE: u32 = 0x30;
}

