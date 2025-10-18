#![feature(impl_trait_in_assoc_type)]
#![allow(unused)]
use core::panic;
use std::marker::PhantomData;

/// Use like a pointer or index
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryHandle<T> {
    generation: u64,
    index: usize,
    enforce_typing: PhantomData<T>
}

#[derive(Debug)]
struct GenVecEntry<T> {
    generation: u64, // even means filled, odd means empty
    data: T,
}

/// Use like a vec
#[derive(Debug)]
pub struct GenVec<T> {
    vec: Vec<GenVecEntry<T>>,
    freelist: Vec<usize>,
}

// macro_rules! mkgetter {
//     ($name:ident $(, $reftype:tt)?) => {
//          pub fn $name(&$($reftype)? self, h: EntryHandle<T>) -> Option<&$($reftype)? T> {
//             if self.vec[h.index].generation != h.generation {
//                 return None;
//             }
//             return Some(&$($reftype)? self.vec[h.index].data);
//         }
//     };
// }

impl<T> GenVec<T> {
    pub fn new() -> Self {
        Self::with_capacity(8)
    }
    pub fn with_capacity(capacity: usize) -> Self {
        GenVec {
            vec: Vec::with_capacity(capacity),
            freelist: Vec::new(),
        }
    }
    /// Allocate a new element, set its initial value (data),
    /// and get a handle to it.
    pub fn alloc(&mut self, data: T) -> EntryHandle<T> {
        let index;
        let generation;
        if let Some(index_) = self.freelist.pop() {
            index = index_;
            generation = self.vec[index].generation;
            self.vec[index].data = data;
        } else {
            index = self.vec.len();
            generation = 0;
            self.vec.push(GenVecEntry { generation, data });
        }
        return EntryHandle {
            generation,
            index,
            enforce_typing: PhantomData
        };
    }
    /// Mark an element as disused. This does not call drop().
    /// This invalidates the handle. Using the handle with
    /// the index_??? functions will panic. Using it with the
    /// get_ functions yields None.
    pub fn free(&mut self, h: EntryHandle<T>) {
        // Increase generation, add to free list
        let el = &mut self.vec[h.index];
        if el.generation != h.generation {
            // panic!("Double free: {:?}", (h.generation, h.index));
            // eprintln!("Double free: {:?}", (h.generation, h.index));
            return;
        }
        el.generation += 1;
        self.freelist.push(h.index);
    }
    /// Safely check if element exists.
    pub fn exists(&self, h: EntryHandle<T>) -> bool {
        if let Some(el) = self.vec.get(h.index) {
            if el.generation == h.generation {
                return true;
            }
        }
        return false;
    }
    /// Get a &T or panic.
    pub fn index_ref(&self, h: EntryHandle<T>) -> &T {
        let el = &self.vec[h.index];
        if el.generation != h.generation {
            panic!("Invalid handle: {:?}", (h.generation, h.index));
        }
        return &el.data;
    }
    /// Get a &mut T or panic.
    pub fn index_mut(&mut self, h: EntryHandle<T>) -> &mut T {
        let el = &mut self.vec[h.index];
        if el.generation != h.generation {
            panic!("Invalid handle: {:?}", (h.generation, h.index));
        }
        return &mut el.data;
    }
    // mkgetter!(get_mut, mut);
    // mkgetter!(get_ref);
    
    /// Get a Some(&T) or None.
    pub fn get_ref(&self, h: EntryHandle<T>) -> Option<&T> {
        if self.vec[h.index].generation != h.generation {
            return None;
        }
        return Some(&self.vec[h.index].data);
    }
    /// Get a Some(&mut T) or None.
    pub fn get_mut(&mut self, h: EntryHandle<T>) -> Option<&mut T> {
        if self.vec[h.index].generation != h.generation {
            return None;
        }
        return Some(&mut self.vec[h.index].data);
    }
    
    /// Get an iterator yields &items.
    /// O(n) over highest number of elements ever in use, not counting underlying vec unused capacity.
    pub fn iter(&self) -> impl Iterator<Item=&T> + '_ {
        self.vec.iter()
            .filter_map(
                |item| ((item.generation & 1) == 0).then_some(&item.data)
            )
    }
    /// Get an iterator yields &mut items.
    /// O(n) over highest number of elements ever in use, not counting underlying vec unused capacity.
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> + '_ {
        self.vec.iter_mut()
            .filter_map(
                |item| ((item.generation & 1) == 0).then_some(&mut item.data)
            )
    }
}

impl <T: Copy> GenVec<T> {
    /// Get a copy of T or panic.
    pub fn index_copy(&self, h: EntryHandle<T>) -> T {
        let el = &self.vec[h.index];
        if el.generation != h.generation {
            panic!("Invalid handle: {:?}", (h.generation, h.index));
        }
        return el.data;
    }
    /// Get a Some(copy of T) or None.
    pub fn get_copy(&self, h: EntryHandle<T>) -> Option<T> {
        if self.vec[h.index].generation != h.generation {
            return None;
        }
        return Some(self.vec[h.index].data);
    }
}

impl<'a, T> IntoIterator for &'a GenVec<T> {
    type Item = &'a T;
    type IntoIter = impl Iterator<Item=&'a T> + 'a;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut GenVec<T> {
    type Item = &'a mut T;
    type IntoIter = impl Iterator<Item=&'a mut T> + 'a;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Default for GenVec<T> {
    fn default() -> Self {
        Self::new()
    }
}


