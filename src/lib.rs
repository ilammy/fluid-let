// Copyright (c) 2019, ilammy
// Licensed under MIT license (see LICENSE)

//! Dynamically-scoped variables.
//!
//! _Dynamic_ or _fluid_ variables are a handy way to define global configuration values.
//! They come from the Lisp family of languages where they are relatively popular in this role.
//!
//! # Declaring dynamic variables
//!
//! [`fluid_let!`] macro is used to declare dynamic variables. Dynamic variables
//! are _global_, therefore they must be declared as `static`:
//!
//! ```
//! use std::fs::File;
//!
//! use fluid_let::fluid_let;
//!
//! fluid_let!(static LOG_FILE: File);
//! ```
//!
//! The actual type of `LOG_FILE` variable will be `Option<&File>`: that is,
//! possibly absent reference to a file. All dynamic variables have `None` as
//! their default value, unless a particular value is set for them.
//!
//! [`fluid_let!`]: macro.fluid_let.html
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
//! # fluid_let!(static LOG_FILE: File);
//! #
//! # fn open(path: &str) -> File { unimplemented!() }
//! #
//! let log_file: File = open("/tmp/log.txt");
//!
//! LOG_FILE.set(&log_file, || {
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
//! The new value is in effect within the _dynamic extent_ of the assignment, that is within
//! the closure passed to `set`. Once the closure returns, the previous value of the variable
//! is restored. You can nest assignments arbitrarily:
//!
//! ```no_run
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: File);
//! #
//! # fn open(path: &str) -> File { unimplemented!() }
//! #
//! LOG_FILE.set(&open("/tmp/log.txt"), || {
//!     //
//!     // log to /tmp/log.txt here
//!     //
//!     LOG_FILE.set(&open("/dev/null"), || {
//!         //
//!         // log to /dev/null for a bit
//!         //
//!     });
//!     //
//!     // log to /tmp/log.txt again
//!     //
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
//! # fluid_let!(static LOG_FILE: File);
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
//! # Thread safety
//!
//! Dynamic variables are global and _thread-local_. That is, each thread gets its own independent
//! instance of a dynamic variable. Values set in one thread are visible only in this thread.
//! Other threads will not see any changes in values of their dynamic variables and may have
//! completely different configurations.
//!
//! Note, however, that this does not free you from the usual synchronization concerns when shared
//! objects are involved. Dynamic variables hold _references_ to objects. Therefore is is entirely
//! possible to bind _the same_ object to a dynamic variable and access it from multiple threads.
//! In this case you will probably need some synchronization to use the shared object in a safe
//! manner, just like you would do when using `Arc` or something.

use std::cell::UnsafeCell;
use std::thread::LocalKey;

/// Declares global dynamic variables.
///
/// # Examples
///
/// One-line form for single declarations:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let!(static ENABLED: bool);
/// ```
///
/// Multiple declarations with attributes and visibility modifiers are also supported:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let! {
///     /// Length of `Debug` representation of hashes in characters.
///     pub static HASH_LENGTH: usize;
///
///     /// If set to true then passwords will be printed to logs.
///     #[cfg(test)]
///     static DUMP_PASSWORDS: bool;
/// }
/// ```
///
/// See also [crate-level documentation](index.html) for usage examples.
#[macro_export]
macro_rules! fluid_let {
    // Simple case: a single definition.
    {
        $(#[$attr:meta])*
        $v:vis static $name:ident: $type_:ty
    } => {
        $(#[$attr])*
        $v static $name: $crate::DynamicVariable<$type_> = {
            // We have to work around the stupid API of thread-local variables in Rust.
            // Hence this atrocity for initialization.
            thread_local! {
                static VARIABLE: $crate::DynamicCell<$type_> = $crate::DynamicCell::empty();
            }
            $crate::DynamicVariable { cell: &VARIABLE }
        };
    };
    // Multiple definitions (iteration).
    {
        $(#[$attr:meta])*
        $v:vis static $name:ident: $type_:ty;
        $($rest:tt)*
    } => {
        $crate::fluid_let!($(#[$attr])* $v static $name: $type_);
        $crate::fluid_let!($($rest)*);
    };
    // No definitions (recursion base).
    {} => {};
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
    cell: UnsafeCell<Option<*const T>>,
}

/// Guard setting a new value of `DynamicCell<T>`.
#[doc(hidden)]
pub struct DynamicCellGuard<'a, T> {
    old_value: Option<*const T>,
    cell: &'a DynamicCell<T>,
}

impl<T> DynamicVariable<T> {
    /// Access current value of the dynamic variable.
    pub fn get<R>(&self, f: impl FnOnce(Option<&T>) -> R) -> R {
        self.cell.with(|current| {
            // This is safe usage when paired with set().
            f(unsafe { current.get() })
        })
    }

    /// Bind a new value to the dynamic variable.
    pub fn set<R>(&self, value: &T, f: impl FnOnce() -> R) -> R {
        self.cell.with(|current| {
            // This is safe usage when paired with get().
            let _guard = unsafe { current.set(value) };
            f()
        })
    }
}

impl<T> DynamicCell<T> {
    /// Makes a new empty cell.
    pub fn empty() -> Self {
        DynamicCell {
            cell: UnsafeCell::new(None),
        }
    }

    /// Access the current value of the cell, if any.
    ///
    /// # Safety
    ///
    /// The returned reference is safe to use during the dynamic extent of a corresponding guard
    /// returned by a `set()` call. Ensure that this reference does not outlive the set value.
    unsafe fn get(&self) -> Option<&T> {
        (&*self.cell.get()).map(|p| &*p)
    }

    /// Temporary set a new value of the cell.
    ///
    /// The value will be active while the returned guard object is live. It will be reset back
    /// to the original value when the guard is dropped.
    ///
    /// # Safety
    ///
    /// You have to ensure that the guard for the previous value is dropped after this one.
    unsafe fn set(&self, value: &T) -> DynamicCellGuard<T> {
        DynamicCellGuard {
            old_value: std::mem::replace(&mut *self.cell.get(), Some(value)),
            cell: self,
        }
    }
}

impl<'a, T> Drop for DynamicCellGuard<'a, T> {
    fn drop(&mut self) {
        // We can safely drop the new value of a cell and restore the old one provided that get()
        // set() methods of DynamicCell are used correctly. That is, there are no users of the
        // new value (which is about to be destroyed).
        unsafe {
            std::mem::replace(&mut *self.cell.cell.get(), self.old_value.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_set_get_guards() {
        // This is how properly scoped usage of DynamicCell works.
        unsafe {
            let v = DynamicCell::empty();
            assert_eq!(v.get(), None);
            {
                let _g = v.set(&5);
                assert_eq!(v.get(), Some(&5));
                {
                    let _g = v.set(&10);
                    assert_eq!(v.get(), Some(&10));
                }
                assert_eq!(v.get(), Some(&5));
            }
        }
    }

    #[test]
    fn cell_unsafe_set_get_usage() {
        // The following is safe because references to constants are 'static,
        // but it is not safe in general case allowed by the API.
        unsafe {
            let v = DynamicCell::empty();
            let g1 = v.set(&5);
            let g2 = v.set(&10);
            assert_eq!(v.get(), Some(&10));
            // Specifically, you CANNOT do this:
            drop(g1);
            // g1 *must* outlive g2 or else you'll that values are restored in
            // incorrect order. Here we observe the value before "5" was set.
            assert_eq!(v.get(), None);
            // When g2 gets dropped it restores the value set by g1, which
            // may not be a valid reference at this point.
            drop(g2);
            assert_eq!(v.get(), Some(&5));
            // And now there's no one to reset the variable to None state.
        }
    }
}
