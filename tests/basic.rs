// Copyright (c) 2019, ilammy
// Licensed under MIT license (see LICENSE)

use std::thread;

use fluid_let::{fluid_let, fluid_set};

#[test]
fn dynamic_scoping() {
    fluid_let!(static YEAR: i32);

    YEAR.get(|current| assert_eq!(current, None));

    fluid_set!(YEAR, &2019);

    YEAR.get(|current| assert_eq!(current, Some(&2019)));
    {
        fluid_set!(YEAR, &2525);

        YEAR.get(|current| assert_eq!(current, Some(&2525)));
    }
    YEAR.get(|current| assert_eq!(current, Some(&2019)));
}

#[test]
fn thread_locality() {
    fluid_let!(static THREAD_ID: i8);

    THREAD_ID.set(&0, || {
        THREAD_ID.get(|current| assert_eq!(current, Some(&0)));
        let t = thread::spawn(move || {
            THREAD_ID.get(|current| assert_eq!(current, None));
            THREAD_ID.set(&1, || {
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

#[test]
fn convenience_accessors() {
    fluid_let!(static ENABLED: bool);

    assert_eq!(ENABLED.cloned(), None);
    assert_eq!(ENABLED.copied(), None);

    ENABLED.set(&true, || assert_eq!(ENABLED.cloned(), Some(true)));
    ENABLED.set(&true, || assert_eq!(ENABLED.copied(), Some(true)));
}
