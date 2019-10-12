// Copyright (c) 2019, ilammy
// Licensed under MIT license (see LICENSE)

//! Dynamically scoped variables.
//!
//! _Dynamic_ or _fluid_ variables are a handy way to define global configuration values.
//! They come from the Lisp family of languages where they are relatively popular in this role.
//!
//! # Declaring dynamic variables
//!
//! [`fluid_let!`] macro is used to declare dynamic variables. Dynamic variables
//! are _global_, therefore they must be declared as `static`:
//!
//! [`fluid_let!`]: macro.fluid_let.html
//!
//! ```
//! use std::fs::File;
//!
//! use fluid_let::fluid_let;
//!
//! fluid_let!(static LOG_FILE: Option<File> = None);
//! ```
//!
//! You also have to provide an initial value for the variable. Since it is static,
//! complex types like `File` may need runtime initialization. Here we use `Option`
//! to initialize the variable statically to a placeholder value.
//!
//! # Setting dynamic variables
//!
//! [`set`] is used to give value to a dynamic variable:
//!
//! [`set`]: struct.DynamicVariable.html#method.set
//!
//! ```no_run
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: Option<File> = None);
//! #
//! # fn open(path: &str) -> File { unimplemented!() }
//! #
//! let log_file: File = open("/tmp/log.txt");
//!
//! LOG_FILE.set(Some(log_file), || {
//!     //
//!     // logs will be redirected to /tmp/log.txt in this block
//!     //
//! });
//! ```
//!
//! Note that you store an _immutable reference_ in the dynamic variable.
//! You can’t directly modify the dynamic variable value after setting it,
//! but you can use something like `Cell` or `RefCell` to circumvent that.
//!
//! The new value is in effect within the _dynamic extent_ of the assignment,
//! that is within the closure passed to `set`. Once the closure returns, the
//! previous value of the variable is restored.
//!
//! If you do not need precise control over the extent of the assignment, you
//! can use the [`fluid_set!`] macro to assign until the end of the scope:
//!
//! [`fluid_set!`]: macro.fluid_set.html
//!
//! ```no_run
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: Option<File> = None);
//! #
//! # fn open(path: &str) -> File { unimplemented!() }
//! #
//! use fluid_let::fluid_set;
//!
//! fn chatterbox_function() {
//!     fluid_set!(LOG_FILE, Some(open("/dev/null")));
//!     //
//!     // logs will be written to /dev/null in this function
//!     //
//! }
//! ```
//!
//! Obviously, you can also nest assignments arbitrarily:
//!
//! ```no_run
//! # use std::fs::File;
//! #
//! # use fluid_let::{fluid_let, fluid_set};
//! #
//! # fluid_let!(static LOG_FILE: Option<File> = None);
//! #
//! # fn open(path: &str) -> File { unimplemented!() }
//! #
//! LOG_FILE.set(Some(open("A.txt")), || {
//!     // log to A.txt here
//!     LOG_FILE.set(Some(open("/dev/null")), || {
//!         // log to /dev/null for a bit
//!         fluid_set!(LOG_FILE, Some(open("B.txt")));
//!         // log to B.txt starting with this line
//!         {
//!             fluid_set!(LOG_FILE, Some(open("C.txt")));
//!             // but in this block log to C.txt
//!         }
//!         // before going back to using B.txt here
//!     });
//!     // and logging to A.txt again
//! });
//! ```
//!
//! # Accessing dynamic variables
//!
//! [`get`] is used to retrieve the current value of a dynamic variable:
//!
//! [`get`]: struct.DynamicVariable.html#method.get
//!
//! ```no_run
//! # use std::io::{self, Write};
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: Option<File> = None);
//! #
//! fn write_log(msg: &str) -> io::Result<()> {
//!     LOG_FILE.get(|current| {
//!         if let Some(mut log_file) = current {
//!             write!(log_file, "{}\n", msg)?;
//!         }
//!         Ok(())
//!     })
//! }
//! ```
//!
//! Current value of the dynamic variable is passed to the provided closure, and
//! the value returned by the closure becomes the value of the `get()` call.
//!
//! This somewhat weird access interface is dictated by safety requirements. The
//! dynamic variable itself is global and thus has `'static` lifetime. However,
//! its values usually have shorter lifetimes, as short as the corresponing
//! `set()` call. Therefore, access reference must have _even shorter_ lifetime.
//!
//! If the variable type implements `Clone` or `Copy` then you can use [`cloned`]
//! and [`copied`] convenience accessors to get a copy of the current value:
//!
//! [`cloned`]: struct.DynamicVariable.html#method.cloned
//! [`copied`]: struct.DynamicVariable.html#method.copied
//!
//! ```no_run
//! # use std::io::{self, Write};
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: Option<File> = None);
//! #
//! #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! enum LogLevel {
//!     Debug,
//!     Info,
//!     Error,
//! }
//!
//! fluid_let!(static MIN_LOG_LEVEL: LogLevel = LogLevel::Info);
//!
//! fn write_log(level: LogLevel, msg: &str) -> io::Result<()> {
//!     if level < MIN_LOG_LEVEL.copied() {
//!         return Ok(());
//!     }
//!     LOG_FILE.get(|current| {
//!         if let Some(ref log_file) = current {
//!             write!(log_file, "{}\n", msg)?;
//!         }
//!         Ok(())
//!     })
//! }
//! ```
//!
//! # Thread safety
//!
//! Dynamic variables are global and _thread-local_. That is, each thread gets
//! its own independent instance of a dynamic variable. Values set in one thread
//! are visible only in this thread. Other threads will not see any changes in
//! values of their dynamic variables and may have different configurations.
//!
//! Note, however, that this does not free you from the usual synchronization
//! concerns when shared objects are involved. Dynamic variables hold _references_
//! to objects. Therefore it is entirely possible to bind _the same_ object with
//! internal mutability to a dynamic variable and access it from multiple threads.
//! In this case you will probably need some synchronization to use the shared
//! object in a safe manner, just like you would do when using `Arc` and friends.

use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::mem;
use std::thread::LocalKey;

/// Declares global dynamic variables.
///
/// # Examples
///
/// One-line form for single declarations:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let!(static ENABLED: bool = true);
/// ```
///
/// Multiple declarations with attributes and visibility modifiers are also supported:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let! {
///     /// Length of `Debug` representation of hashes in characters.
///     pub static HASH_LENGTH: usize = 32;
///
///     /// If set to true then passwords will be printed to logs.
///     #[cfg(test)]
///     static DUMP_PASSWORDS: bool = false;
/// }
/// ```
///
/// See also [crate-level documentation](index.html) for usage examples.
#[macro_export]
macro_rules! fluid_let {
    // Simple case: a single definition.
    {
        $(#[$attr:meta])*
        $v:vis static $name:ident: $type_:ty = $value:expr
    } => {
        $(#[$attr])*
        $v static $name: $crate::DynamicVariable<$type_> = {
            static DEFAULT: $type_ = $value;
            thread_local! {
                static VARIABLE: $crate::DynamicCell<$type_> = $crate::DynamicCell::with_static(&DEFAULT);
            }
            $crate::DynamicVariable { cell: &VARIABLE }
        };
    };
    // Multiple definitions (iteration).
    {
        $(#[$attr:meta])*
        $v:vis static $name:ident: $type_:ty = $value:expr;
        $($rest:tt)*
    } => {
        $crate::fluid_let!($(#[$attr])* $v static $name: $type_ = $value);
        $crate::fluid_let!($($rest)*);
    };
    // No definitions (recursion base).
    {} => {};
}

/// Binds a value to a dynamic variable.
///
/// # Examples
///
/// If you do not need to explicitly delimit the scope of dynamic assignment then you can
/// use `fluid_set!` to assign a value until the end of the current scope:
///
/// ```no_run
/// use fluid_let::{fluid_let, fluid_set};
///
/// fluid_let!(static ENABLED: bool = false);
///
/// fn some_function() {
///     fluid_set!(ENABLED, &true);
///
///     // function body
/// }
/// ```
///
/// This is effectively equivalent to writing
///
/// ```no_run
/// # use fluid_let::{fluid_let, fluid_set};
/// #
/// # fluid_let!(static ENABLED: bool = false);
/// #
/// fn some_function() {
///     ENABLED.set(&true, || {
///         // function body
///     });
/// }
/// ```
///
/// See also [crate-level documentation](index.html) for usage examples.
#[macro_export]
macro_rules! fluid_set {
    ($variable:expr, $value:expr) => {
        let _value_ = $value;
        // This is safe because the users do not get direct access to the guard
        // and are not able to drop it prematurely, thus maintaining invariants.
        let _guard_ = unsafe { $variable.set_guard(&_value_) };
    };
}

/// A global dynamic variable.
///
/// Declared and initialized by the [`fluid_let!`](macro.fluid_let.html) macro.
///
/// See [crate-level documentation](index.html) for examples.
pub struct DynamicVariable<T: 'static> {
    #[doc(hidden)]
    pub cell: &'static LocalKey<DynamicCell<T>>,
}

/// A resettable reference.
#[doc(hidden)]
pub struct DynamicCell<T> {
    cell: UnsafeCell<*const T>,
}

/// Guard setting a new value of `DynamicCell<T>`.
#[doc(hidden)]
pub struct DynamicCellGuard<'a, T> {
    old_value: *const T,
    cell: &'a DynamicCell<T>,
}

impl<T> DynamicVariable<T> {
    /// Access current value of the dynamic variable.
    pub fn get<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.cell.with(|current| {
            // This is safe because the lifetime of the reference returned by get()
            // is limited to this block so it cannot outlive any value set by set()
            // in the caller frames.
            f(unsafe { current.get() })
        })
    }

    /// Bind a new value to the dynamic variable.
    pub fn set<R>(&self, value: impl Borrow<T>, f: impl FnOnce() -> R) -> R {
        self.cell.with(|current| {
            // This is safe because the guard returned by set() is guaranteed to be
            // dropped after the thunk returns and before anything else executes.
            let _guard_ = unsafe { current.set(value.borrow()) };
            f()
        })
    }

    /// Bind a new value to the dynamic variable.
    ///
    /// # Safety
    ///
    /// The value is bound for the lifetime of the returned guard. The guard must be
    /// dropped before the end of lifetime of the new and old assignment values.
    /// If the variable is assigned another value while this guard is alive, it must
    /// not be dropped until that new assignment is undone.
    #[doc(hidden)]
    pub unsafe fn set_guard(&self, value: &T) -> DynamicCellGuard<T> {
        // We use transmute to extend the lifetime or "current" to that of "value".
        // This is really the case when assignments are properly scoped.
        unsafe fn extend_lifetime<'a, 'b, T>(r: &'a T) -> &'b T {
            mem::transmute(r)
        }
        self.cell
            .with(|current| extend_lifetime(current).set(value))
    }
}

impl<T: Clone> DynamicVariable<T> {
    /// Clone current value of the dynamic variable.
    pub fn cloned(&self) -> T {
        self.get(|value| value.clone())
    }
}

impl<T: Copy> DynamicVariable<T> {
    /// Copy current value of the dynamic variable.
    pub fn copied(&self) -> T {
        self.get(|value| *value)
    }
}

impl<T> DynamicCell<T> {
    /// Makes a new cell with value.
    pub fn with_static(value: &'static T) -> Self {
        DynamicCell {
            cell: UnsafeCell::new(value),
        }
    }

    /// Access the current value of the cell, if any.
    ///
    /// # Safety
    ///
    /// The returned reference is safe to use during the lifetime of a corresponding guard
    /// returned by a `set()` call. Ensure that this reference does not outlive it.
    unsafe fn get(&self) -> &T {
        &**self.cell.get()
    }

    /// Temporarily set a new value of the cell.
    ///
    /// The value will be active while the returned guard object is live. It will be reset
    /// back to the original value (at the moment of the call) when the guard is dropped.
    ///
    /// # Safety
    ///
    /// You have to ensure that the guard for the previous value is dropped after this one.
    /// That is, they must be dropped in strict LIFO order, like a call stack.
    unsafe fn set(&self, value: &T) -> DynamicCellGuard<T> {
        DynamicCellGuard {
            old_value: mem::replace(&mut *self.cell.get(), value),
            cell: self,
        }
    }
}

impl<'a, T> Drop for DynamicCellGuard<'a, T> {
    fn drop(&mut self) {
        // We can safely drop the new value of a cell and restore the old one provided that
        // get() and set() methods of DynamicCell are used correctly. That is, there must be
        // no users of the new value which is about to be destroyed.
        unsafe {
            *self.cell.cell.get() = self.old_value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt;
    use std::thread;

    #[test]
    fn cell_set_get_guards() {
        // This is how properly scoped usage of DynamicCell works.
        unsafe {
            let v = DynamicCell::with_static(&0);
            assert_eq!(v.get(), &0);
            {
                let _g = v.set(&5);
                assert_eq!(v.get(), &5);
                {
                    let _g = v.set(&10);
                    assert_eq!(v.get(), &10);
                }
                assert_eq!(v.get(), &5);
            }
        }
    }

    #[test]
    fn cell_unsafe_set_get_usage() {
        // The following is safe because references to constants are 'static,
        // but it is not safe in general case allowed by the API.
        unsafe {
            let v = DynamicCell::with_static(&0);
            let g1 = v.set(&5);
            let g2 = v.set(&10);
            assert_eq!(v.get(), &10);
            // Specifically, you CANNOT do this:
            drop(g1);
            // g1 *must* outlive g2 or else you'll that values are restored in
            // incorrect order. Here we observe the value before "5" was set.
            assert_eq!(v.get(), &0);
            // When g2 gets dropped it restores the value set by g1, which
            // may not be a valid reference at this point.
            drop(g2);
            assert_eq!(v.get(), &5);
            // And now there's no one to reset the variable to None state.
        }
    }

    #[test]
    fn static_initializer() {
        fluid_let!(static NUMBER: i32 = 42);

        assert_eq!(NUMBER.copied(), 42);

        fluid_let! {
            static NUMBER_1: i32 = 100;
            static NUMBER_2: i32 = 200;
            static NUMBER_3: i32 = 300;
        }

        assert_eq!(NUMBER_1.copied(), 100);
        assert_eq!(NUMBER_2.copied(), 200);
        assert_eq!(NUMBER_3.copied(), 300);
    }

    #[test]
    fn dynamic_scoping() {
        fluid_let!(static YEAR: i32 = 1986);

        YEAR.get(|current| assert_eq!(current, &1986));

        fluid_set!(YEAR, 2019);

        YEAR.get(|current| assert_eq!(current, &2019));
        {
            fluid_set!(YEAR, 2525);

            YEAR.get(|current| assert_eq!(current, &2525));
        }
        YEAR.get(|current| assert_eq!(current, &2019));
    }

    #[test]
    fn references() {
        fluid_let!(static YEAR: i32 = -1);

        // Temporary value
        fluid_set!(YEAR, 10);
        assert_eq!(YEAR.copied(), 10);

        // Local reference
        let current_year = 20;
        fluid_set!(YEAR, &current_year);
        assert_eq!(YEAR.copied(), 20);

        // Heap reference
        let current_year = Box::new(30);
        fluid_set!(YEAR, current_year);
        assert_eq!(YEAR.copied(), 30);
    }

    #[test]
    fn thread_locality() {
        fluid_let!(static THREAD_ID: i8 = -1);

        THREAD_ID.set(0, || {
            THREAD_ID.get(|current| assert_eq!(current, &0));
            let t = thread::spawn(move || {
                THREAD_ID.get(|current| assert_eq!(current, &-1));
                THREAD_ID.set(1, || {
                    THREAD_ID.get(|current| assert_eq!(current, &1));
                });
            });
            drop(t.join());
        })
    }

    #[test]
    fn convenience_accessors() {
        fluid_let!(static ENABLED: bool = false);

        assert_eq!(ENABLED.cloned(), false);
        assert_eq!(ENABLED.copied(), false);

        ENABLED.set(true, || assert_eq!(ENABLED.cloned(), true));
        ENABLED.set(true, || assert_eq!(ENABLED.copied(), true));
    }

    struct Hash {
        value: [u8; 16],
    }

    fluid_let!(pub static DEBUG_FULL_HASH: bool = false);

    impl fmt::Debug for Hash {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let full = DEBUG_FULL_HASH.copied();

            write!(f, "Hash(")?;
            if full {
                for byte in &self.value {
                    write!(f, "{:02X}", byte)?;
                }
            } else {
                for byte in &self.value[..4] {
                    write!(f, "{:02X}", byte)?;
                }
                write!(f, "...")?;
            }
            write!(f, ")")
        }
    }

    #[test]
    fn readme_example_code() {
        let hash = Hash { value: [0; 16] };
        assert_eq!(format!("{:?}", hash), "Hash(00000000...)");
        fluid_set!(DEBUG_FULL_HASH, true);
        assert_eq!(
            format!("{:?}", hash),
            "Hash(00000000000000000000000000000000)"
        );
    }
}
