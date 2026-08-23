use crate::types::{Pointer, RuntimeValue, Storage};

#[derive(Clone, Debug)]
pub struct MemoryStore {
    pub id: u32,
    pub storage: Storage,
    pub objects: Vec<RuntimeValue>,
}

impl MemoryStore {
    pub fn new(storage: Storage) -> Self {
        MemoryStore {
            storage,
            id: rand::random(),
            objects: Vec::new(),
        }
    }

    pub fn alloc(&mut self, init: Option<RuntimeValue>) -> Pointer {
        let index = self.objects.len();
        if let Some(init) = init {
            self.objects.push(init);
        } else {
            self.objects.push(RuntimeValue::Null);
        }

        Pointer {
            storage_id: self.id,
            id: index.into(),
            offsets: Vec::new(),
        }
    }

    fn mem_read_inner(&self, mut offsets: Vec<usize>, value: RuntimeValue) -> Option<RuntimeValue> {
        if offsets.is_empty() {
            return Some(value);
        }
        let offset = offsets.remove(0);
        match value {
            RuntimeValue::Vec { lenght, contents } => {
                self.mem_read_inner(offsets, contents[offset].clone().into())
            }
            RuntimeValue::Struct { members } => {
                self.mem_read_inner(offsets, members[offset].clone())
            }
            _ => panic!(),
        }
    }

    pub fn read(&self, pointer: Pointer) -> Option<RuntimeValue> {
        let index: usize = pointer.id.into();
        match self.objects.get(index) {
            Some(v) => self.mem_read_inner(pointer.offsets.clone(), v.clone()),
            None => None,
        }
    }

    fn mem_modify_inner(
        &self,
        mut offsets: Vec<usize>,
        value: RuntimeValue,
        new_value: RuntimeValue,
    ) -> RuntimeValue {
        if offsets.is_empty() {
            return new_value;
        }
        let offset = offsets.remove(0);
        match value {
            RuntimeValue::Vec { lenght, contents } => {
                let mut contents = contents.clone();
                let val = contents[offset].clone().into();
                contents[offset] = self
                    .mem_modify_inner(offsets, val, new_value)
                    .try_into()
                    .unwrap();
                RuntimeValue::Vec { lenght, contents }
            }

            RuntimeValue::Struct { members } => {
                let mut members = members.clone();
                let value = members[offset].clone();
                members[offset] = self.mem_modify_inner(offsets, value, new_value);
                RuntimeValue::Struct { members }
            }
            _ => panic!(),
        }
    }

    pub fn write(&mut self, pointer: Pointer, value: RuntimeValue) {
        let index: usize = pointer.id.into();
        if index >= self.objects.len() {
            panic!(
                "Writing to object not yet alloced: {index} vs {}",
                self.objects.len()
            );
        }

        if pointer.offsets.is_empty() {
            self.objects[index] = value;
        } else {
            let old_value = self.objects[index].clone();
            self.objects[index] = self.mem_modify_inner(pointer.offsets, old_value, value);
        }
    }
}
