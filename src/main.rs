#![feature(impl_trait_in_assoc_type)]
#![allow(unused)]







pub mod genvec;
use genvec::*;





fn main() {
    let mut vec: GenVec<u32> = GenVec::with_capacity(1000);
    let handle = vec.alloc(777);
    
    let value = *vec.get_mut(handle).unwrap();
    println!("{:?}", value);
    
    let value = *vec.get_ref(handle).unwrap();
    println!("{:?}", value);
    
    let value = vec.get_copy(handle).unwrap();
    println!("{:?}", value);
    
    for i in 1..10 {
        vec.alloc(i);
    }
    
    // Test mutable iterator
    for el in vec.iter_mut() {
        *el += 1;
    }
    for el in &mut vec {
        *el += 1;
    }
    
    assert!(vec.exists(handle));
    vec.free(handle);
    assert!(! vec.exists(handle));
    
    println!("-------------");
    for i in &vec {
        println!("{:?}", i);
    }
    
    println!("{:?}", vec);
    
}
