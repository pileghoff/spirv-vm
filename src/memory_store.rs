use crate::types::{Pointer, RuntimeValue, Storage};

#[derive(Clone, Debug)]
pub struct MemoryStore {
    pub id: u32,
    pub storage: Storage,
    pub objects: Vec<RuntimeValue>,
}

// Todo: This should be a function on RuntimeValue

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

    pub fn read(&self, pointer: Pointer) -> Option<RuntimeValue> {
        let index: usize = pointer.id.into();
        match self.objects.get(index) {
            Some(v) => Some(v.clone().read_inner(pointer.offsets.clone())),
            None => None,
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
            self.objects[index] = old_value.modify_inner(pointer.offsets, value);
        }
    }
}
