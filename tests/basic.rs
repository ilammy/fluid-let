// Copyright (c) 2019, ilammy
// Licensed under MIT license (see LICENSE)

use std::thread;

use fluid_let::fluid_let;

fluid_let!(static YEAR: i32);

#[test]
fn dynamic_scoping() {
    YEAR.get(|current| assert_eq!(current, None));

    YEAR.set(2019, || {
        YEAR.get(|current| assert_eq!(current, Some(&2019)));

        YEAR.set(&2525, || {
            YEAR.get(|current| assert_eq!(current, Some(&2525)));
        })
    });
}

#[test]
fn references() {
    fn get_year() -> i32 {
        YEAR.get(|value| *value.unwrap())
    }

    // Temporary value
    YEAR.set(10, || assert_eq!(get_year(), 10));

    // Local reference
    let current_year = 20;
    YEAR.set(&current_year, || assert_eq!(get_year(), 20));

    // Heap reference
    let current_year = Box::new(30);
    YEAR.set(current_year, || assert_eq!(get_year(), 30));
}

fluid_let!(static THREAD_ID: i8);

#[test]
fn thread_locality() {
    THREAD_ID.set(0, || {
        THREAD_ID.get(|current| assert_eq!(current, Some(&0)));
        let t = thread::spawn(move || {
            THREAD_ID.get(|current| assert_eq!(current, None));
            THREAD_ID.set(1, || {
                THREAD_ID.get(|current| assert_eq!(current, Some(&1)));
            });
        });
        drop(t.join());
    })
}

// Compile-time test for multiple definitions and attributes.
fluid_let! {
    /// Variable 1
    static VAR_1: bool;
    /// Variable 2
    static VAR_2: ();
    /// Variable 3
    pub static VAR_3: u8;
}
