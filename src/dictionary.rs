use crate::bt_memory;
use asr::{Address64, Process};


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

pub struct Dictionary {
    pub address: Address64,
}

impl Dictionary {
    pub fn new(address: Address64) -> Dictionary {
        Dictionary {
            address,
        }
    }

    pub fn get_first_element(&self, process: &Process) -> Option<Element> {
        let first_address = bt_memory::read_pointer(process, self.address + offsets::FIRST)?;
        let first_address = bt_memory::read_pointer(process, first_address + offsets::FIRST_2)?;

        let first_element = Element { address: first_address };

        return Some(first_element);
    }

    pub fn get_length(&self, process: &Process) -> Option<i32> {
        let len = bt_memory::read_int(process, self.address + offsets::SIZE)?;

        return Some(len);
    }

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
}

mod offsets {
    pub const SIZE: u32 = 0x3c;
    pub const FIRST: u32 = 0x18;
    pub const FIRST_2: u32 = 0x50;
}

mod element_offsets {
    pub const NEXT: u32 = 0x0;
    pub const KEY: u32 = 0x18;
    pub const VALUE: u32 = 0x30;
}

