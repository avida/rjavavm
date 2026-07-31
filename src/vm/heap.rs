pub mod heap {
    use std::collections::HashMap;

    use crate::vm::types::types::Type;

    pub type HeapId = usize;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Object {
        pub class_name: String,
        pub fields: HashMap<String, Type>,
    }

    impl Object {
        pub fn new<S: Into<String>>(class_name: S) -> Self {
            Self {
                class_name: class_name.into(),
                fields: HashMap::new(),
            }
        }

        pub fn get_field(&self, name: &str) -> Option<&Type> {
            self.fields.get(name)
        }

        pub fn set_field<S: Into<String>>(&mut self, name: S, value: Type) {
            self.fields.insert(name.into(), value);
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Array {
        pub element_descriptor: String,
        pub elements: Vec<Type>,
    }

    impl Array {
        pub fn new<S: Into<String>>(element_descriptor: S, len: usize) -> Self {
            Self {
                element_descriptor: element_descriptor.into(),
                elements: vec![Type::Null; len],
            }
        }

        pub fn len(&self) -> usize {
            self.elements.len()
        }

        pub fn get(&self, index: usize) -> Option<&Type> {
            self.elements.get(index)
        }

        pub fn set(&mut self, index: usize, value: Type) -> Option<()> {
            let slot = self.elements.get_mut(index)?;
            *slot = value;
            Some(())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct InternalString {
        pub value: String,
    }

    impl InternalString {
        pub fn new<S: Into<String>>(value: S) -> Self {
            Self {
                value: value.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum HeapEntry {
        Object(Object),
        Array(Array),
        InternalString(InternalString),
    }

    #[derive(Debug, Default)]
    pub struct Heap {
        next_id: HeapId,
        entries: HashMap<HeapId, HeapEntry>,
    }

    impl Heap {
        pub fn new() -> Self {
            Self {
                next_id: 1,
                entries: HashMap::new(),
            }
        }

        pub fn allocate_object<S: Into<String>>(&mut self, class_name: S) -> HeapId {
            self.insert(HeapEntry::Object(Object::new(class_name)))
        }

        pub fn allocate_array<S: Into<String>>(
            &mut self,
            element_descriptor: S,
            len: usize,
        ) -> HeapId {
            self.insert(HeapEntry::Array(Array::new(element_descriptor, len)))
        }

        pub fn allocate_string<S: Into<String>>(&mut self, value: S) -> HeapId {
            self.insert(HeapEntry::InternalString(InternalString::new(value)))
        }

        pub fn get(&self, id: HeapId) -> Option<&HeapEntry> {
            self.entries.get(&id)
        }

        pub fn get_mut(&mut self, id: HeapId) -> Option<&mut HeapEntry> {
            self.entries.get_mut(&id)
        }

        pub fn get_object(&self, id: HeapId) -> Option<&Object> {
            match self.entries.get(&id) {
                Some(HeapEntry::Object(object)) => Some(object),
                _ => None,
            }
        }

        pub fn get_object_mut(&mut self, id: HeapId) -> Option<&mut Object> {
            match self.entries.get_mut(&id) {
                Some(HeapEntry::Object(object)) => Some(object),
                _ => None,
            }
        }

        pub fn get_array(&self, id: HeapId) -> Option<&Array> {
            match self.entries.get(&id) {
                Some(HeapEntry::Array(array)) => Some(array),
                _ => None,
            }
        }

        pub fn get_array_mut(&mut self, id: HeapId) -> Option<&mut Array> {
            match self.entries.get_mut(&id) {
                Some(HeapEntry::Array(array)) => Some(array),
                _ => None,
            }
        }

        pub fn get_string(&self, id: HeapId) -> Option<&InternalString> {
            match self.entries.get(&id) {
                Some(HeapEntry::InternalString(string)) => Some(string),
                _ => None,
            }
        }

        pub fn remove(&mut self, id: HeapId) -> Option<HeapEntry> {
            self.entries.remove(&id)
        }

        pub fn len(&self) -> usize {
            self.entries.len()
        }

        fn insert(&mut self, entry: HeapEntry) -> HeapId {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                self.next_id = 1;
            }
            self.entries.insert(id, entry);
            id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::heap::{Heap, HeapEntry};
    use crate::vm::types::types::Type;

    #[test]
    fn allocates_object_and_sets_field() {
        let mut heap = Heap::new();
        let id = heap.allocate_object("java/io/PrintStream");

        let object = heap.get_object_mut(id).unwrap();
        object.set_field("out:Ljava/lang/String;", Type::Reference(7));

        let object = heap.get_object(id).unwrap();
        assert_eq!(object.class_name, "java/io/PrintStream");
        assert_eq!(
            object.get_field("out:Ljava/lang/String;"),
            Some(&Type::Reference(7))
        );
    }

    #[test]
    fn allocates_array_and_updates_element() {
        let mut heap = Heap::new();
        let id = heap.allocate_array("Ljava/lang/String;", 2);

        let array = heap.get_array_mut(id).unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), Some(&Type::Null));
        assert_eq!(array.set(1, Type::Reference(99)), Some(()));

        let array = heap.get_array(id).unwrap();
        assert_eq!(array.get(1), Some(&Type::Reference(99)));
    }

    #[test]
    fn allocates_internal_string_entry() {
        let mut heap = Heap::new();
        let id = heap.allocate_string("Hello, JVM!");

        let string = heap.get_string(id).unwrap();
        assert_eq!(string.value, "Hello, JVM!");
        assert!(matches!(heap.get(id), Some(HeapEntry::InternalString(_))));
    }

    #[test]
    fn removes_entries() {
        let mut heap = Heap::new();
        let id = heap.allocate_object("java/lang/Object");
        assert_eq!(heap.len(), 1);
        assert!(matches!(heap.remove(id), Some(HeapEntry::Object(_))));
        assert_eq!(heap.len(), 0);
    }
}
