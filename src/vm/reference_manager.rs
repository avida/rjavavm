use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A simple 32-bit reference manager for this VM.
///
/// It hands out unique `u32` references for the lifetime of the current run
/// and stores either a heap-id (an external id from the heap) or a symbolic
/// identifier (for field/method references resolved later).
#[derive(Debug)]
pub struct ReferenceManager {
    next: u32,
    refs: HashMap<u32, ReferenceEntry>,
}

#[derive(Debug, Clone)]
pub enum ReferenceEntry {
    Heap(u32),      // maps to a heap-internal id (now a u32 reference)
    Symbolic(String), // unresolved field/method identifier
}

impl ReferenceManager {
    /// Create a new ReferenceManager. References start at 1; 0 is reserved/null.
    pub fn new() -> ReferenceManager {
        ReferenceManager { next: 1, refs: HashMap::new() }
    }

    /// Allocate a new reference and register it as a heap entry key.
    /// Returns the new `reference_u32`.
    pub fn allocate_new(&mut self) -> u32 {
        let r = self.next;
        self.next = self.next.wrapping_add(1);
        if self.next == 0 {
            self.next = 1;
        }
        // store the heap mapping using the reference itself as the heap id
        self.refs.insert(r, ReferenceEntry::Heap(r));
        r
    }

    /// Allocate a new reference for an existing heap id.
    /// Returns the new 32-bit reference value.
    pub fn allocate_heap(&mut self, heap_id: u32) -> u32 {
        let r = self.next;
        self.next = self.next.wrapping_add(1);
        if self.next == 0 {
            self.next = 1;
        }
        self.refs.insert(r, ReferenceEntry::Heap(heap_id));
        r
    }

    /// Allocate a reference for a symbolic identifier (e.g. "pkg/Class.method()V").
    pub fn allocate_symbolic<S: Into<String>>(&mut self, id: S) -> u32 {
        let r = self.next;
        self.next = self.next.wrapping_add(1);
        if self.next == 0 {
            self.next = 1;
        }
        self.refs.insert(r, ReferenceEntry::Symbolic(id.into()));
        r
    }

    /// Resolve a reference to an entry if present.
    pub fn resolve(&self, reference: u32) -> Option<ReferenceEntry> {
        self.refs.get(&reference).cloned()
    }

    /// Try to resolve a reference as a heap id.
    pub fn resolve_heap(&self, reference: u32) -> Option<u32> {
        match self.refs.get(&reference) {
            Some(ReferenceEntry::Heap(id)) => Some(*id),
            _ => None,
        }
    }

    /// Try to resolve a reference as a symbolic identifier.
    pub fn resolve_symbolic(&self, reference: u32) -> Option<String> {
        match self.refs.get(&reference) {
            Some(ReferenceEntry::Symbolic(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Remove a reference from manager, returning the stored entry if any.
    pub fn remove(&mut self, reference: u32) -> Option<ReferenceEntry> {
        self.refs.remove(&reference)
    }

    /// Number of tracked references.
    pub fn len(&self) -> usize {
        self.refs.len()
    }
}

pub type ReferenceManagerPtr = Arc<Mutex<ReferenceManager>>;

impl ReferenceManager {
    /// Create a shared, thread-safe pointer to a new ReferenceManager
    pub fn new_ptr() -> ReferenceManagerPtr {
        Arc::new(Mutex::new(ReferenceManager::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_resolve_heap() {
        let mut rm = ReferenceManager::new();
        let r = rm.allocate_heap(42u32);
        assert!(r != 0);
        assert_eq!(rm.resolve_heap(r), Some(42u32));
        assert_eq!(rm.len(), 1);
    }

    #[test]
    fn allocate_and_resolve_symbolic() {
        let mut rm = ReferenceManager::new();
        let id = "java/lang/Foo.bar()V".to_string();
        let r = rm.allocate_symbolic(id.clone());
        assert!(r != 0);
        assert_eq!(rm.resolve_symbolic(r), Some(id));
        assert_eq!(rm.len(), 1);
    }

    #[test]
    fn remove_reference() {
        let mut rm = ReferenceManager::new();
        let r1 = rm.allocate_heap(1u32);
        let r2 = rm.allocate_symbolic("X".to_string());
        assert_eq!(rm.len(), 2);
        assert!(matches!(rm.remove(r1), Some(ReferenceEntry::Heap(1u32))));
        assert_eq!(rm.len(), 1);
        assert!(matches!(rm.remove(r2), Some(ReferenceEntry::Symbolic(_))));
        assert_eq!(rm.len(), 0);
    }
}
