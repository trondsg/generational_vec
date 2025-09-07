#![allow(unused)]
use core::panic;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
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

#[derive(Debug)]
pub struct GenVec<T> {
    vec: Vec<GenVecEntry<T>>,
    freelist: Vec<usize>,
}


// macro_rules! mkgetter {
//     ($name:ident, $reftype:tt) => {
//          fn $name(&$reftype self, h: EntryHandle<T>) -> Option<&$reftype T> {
//             if self.vec[h.index].generation != h.generation {
//                 return None;
//             }
//             return Some(&$reftype self.vec[h.index].data);
//         }
//     };
// }


macro_rules! mkgetter {
    ($name:ident $(, $reftype:tt)?) => {
         fn $name(&$($reftype)? self, h: EntryHandle<T>) -> Option<&$($reftype)? T> {
            if self.vec[h.index].generation != h.generation {
                return None;
            }
            return Some(&$($reftype)? self.vec[h.index].data);
        }
    };
}

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
            self.vec.push(GenVecEntry { generation, data: data });
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
    // mkgetter!(get_mut, &mut);
    // mkgetter!(get, &);
        
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
    // mkgetter!(get_mut, mut);
    // mkgetter!(get_ref);
    /// Get an iterator that only yields filled items.
    /// O(n) over highest number of elements ever in use, not counting underlying vec unused capacity.
    pub fn iter(&self) -> GenVecIter<'_, T> {
        GenVecIter {
            container: self,
            index: 0,
        }
    }
}

impl <T: Copy> GenVec<T> {
    /// Get a copy of T or None.
    pub fn get_copy(&self, h: EntryHandle<T>) -> Option<T> {
        if self.vec[h.index].generation != h.generation {
            return None;
        }
        return Some(self.vec[h.index].data);
    }
    /// Get a T or panic.
    pub fn index_copy(&self, h: EntryHandle<T>) -> T {
        let el = &self.vec[h.index];
        if el.generation != h.generation {
            panic!("Invalid handle: {:?}", (h.generation, h.index));
        }
        return el.data;
    }
}

#[allow(non_snake_case)]
fn __Iterators__() {}

pub struct GenVecIter<'a, T> {
    container: &'a GenVec<T>,
    index: usize,
}

impl<'a, T> Iterator for GenVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // case 1: end of vec -> None
        // case 2: found item -> Some(&item)
        // case 3: empty item -> self.index += 1; retry
        loop {
            // case 1
            if self.index == self.container.vec.len() {
                return None;
            }
            // case 2
            let item = &self.container.vec[self.index];
            if item.generation & 1 == 0 {
                self.index += 1;
                return Some(&item.data);
            }
            // case 3
            self.index += 1;
            continue;
        }
    }
}



// impl<T> Default for GenVec<T> {
//     fn default() -> Self {
//         Self::new()
//     }
// }


