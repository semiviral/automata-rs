use std::{
    collections::HashMap,
    sync::{atomic::AtomicU16, RwLock},
};

bitflags::bitflags! {
    pub struct Attributes : u64 {
        const TRANSPARENT = 1 << 0;
        const COLLIDEABLE = 1 << 1;
        const DESCTRUCTIBLE = 1 << 2;
        const COLLECTABLE  = 1 << 3;
    }
}

struct BlockDefinition {
    name: String,
    attribs: Attributes,
}

impl BlockDefinition {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn attribs(&self) -> Attributes {
        self.attribs
    }

    pub fn has_attribute(&self, attrib: Attributes) -> bool {
        self.attribs.contains(attrib)
    }
}

pub struct BlockRegistry {
    definitions: RwLock<Vec<BlockDefinition>>,
    id_lookup: RwLock<HashMap<String, u16>>,
    next_id: AtomicU16,
}

impl BlockRegistry {
    pub const AIR_ID: u16 = 0;

    fn next_id(&self) -> Option<u16> {
        use std::sync::atomic::Ordering;

        if self.next_id.load(Ordering::Acquire) < u16::MAX {
            Some(self.next_id.fetch_add(1, Ordering::AcqRel))
        } else {
            None
        }
    }

    pub fn register_block(&self, group: &str, name: &str, attribs: Attributes) -> u16 {
        let definition = BlockDefinition {
            name: format!("{}:{}", group, name),
            attribs,
        };

        let id = self.next_id().expect("Out of valid block IDs!");
        debug!(
            "Registering block definition: \"{}\": {}",
            definition.name(),
            id,
        );
        self.id_lookup
            .write()
            .unwrap()
            .insert(definition.name.clone(), id);
        self.definitions.write().unwrap().push(definition);

        id
    }

    pub fn block_exists(&self, id: u16) -> bool {
        (id as usize) < self.definitions.read().unwrap().len()
    }

    pub fn get_block_id(&self, name: String) -> Option<u16> {
        self.id_lookup.read().unwrap().get(&name).copied()
    }

    pub fn get_block_name(&self, id: u16) -> String {
        self.definitions.read().unwrap()[id as usize]
            .name()
            .to_string()
    }

    pub fn get_block_attributes(&self, id: u16) -> Attributes {
        self.definitions.read().unwrap()[id as usize].attribs()
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        let registry = BlockRegistry {
            definitions: RwLock::new(Vec::new()),
            id_lookup: RwLock::new(HashMap::new()),
            next_id: AtomicU16::new(0),
        };

        registry.register_block("core", "air", Attributes::TRANSPARENT);

        registry
    }
}
