/// A macro for defining #[cfg] if-else statements.
///
/// This is similar to the `if/elif` C preprocessor macro by allowing definition
/// of a cascade of `#[cfg]` cases, emitting the implementation which matches
/// first.
///
/// This allows you to conveniently provide a long list #[cfg]'d blocks of code
/// without having to rewrite each clause multiple times.
macro_rules! cfg_if {
    // match if/else chains with a final `else`
    ($(
        if #[cfg($($meta:meta),*)] { $($it:item)* }
    ) else * else {
        $($it2:item)*
    }) => {
        cfg_if! {
            @__items
            () ;
            $( ( ($($meta),*) ($($it)*) ), )*
            ( () ($($it2)*) ),
        }
    };

    // match if/else chains lacking a final `else`
    (
        if #[cfg($($i_met:meta),*)] { $($i_it:item)* }
        $(
            else if #[cfg($($e_met:meta),*)] { $($e_it:item)* }
        )*
    ) => {
        cfg_if! {
            @__items
            () ;
            ( ($($i_met),*) ($($i_it)*) ),
            $( ( ($($e_met),*) ($($e_it)*) ), )*
            ( () () ),
        }
    };

    // Internal and recursive macro to emit all the items
    //
    // Collects all the negated `cfg`s in a list at the beginning and after the
    // semicolon is all the remaining items
    (@__items ($($not:meta,)*) ; ) => {};
    (@__items ($($not:meta,)*) ; ( ($($m:meta),*) ($($it:item)*) ),
     $($rest:tt)*) => {
        // Emit all items within one block, applying an appropriate #[cfg]. The
        // #[cfg] will require all `$m` matchers specified and must also negate
        // all previous matchers.
        cfg_if! { @__apply cfg(all($($m,)* not(any($($not),*)))), $($it)* }

        // Recurse to emit all other items in `$rest`, and when we do so add all
        // our `$m` matchers to the list of `$not` matchers as future emissions
        // will have to negate everything we just matched as well.
        cfg_if! { @__items ($($not,)* $($m,)*) ; $($rest)* }
    };

    // Internal macro to Apply a cfg attribute to a list of items
    (@__apply $m:meta, $($it:item)*) => {
        $(#[$m] $it)*
    };
}

/// Implement `Clone` and `Copy` for a struct, as well as `Debug`, `Eq`, `Hash`, and
/// `PartialEq` if the `extra_traits` feature is enabled.
///
/// Use [`s_no_extra_traits`] for structs where the `extra_traits` feature does not
/// make sense, and for unions.
macro_rules! s {
    ($(
        $(#[$attr:meta])*
        pub $t:ident $i:ident { $($field:tt)* }
    )*) => ($(
        s!(it: $(#[$attr])* pub $t $i { $($field)* });
    )*);

    (it: $(#[$attr:meta])* pub union $i:ident { $($field:tt)* }) => (
        compile_error!("unions cannot derive extra traits, use s_no_extra_traits instead");
    );

    (it: $(#[$attr:meta])* pub struct $i:ident { $($field:tt)* }) => (
        __item! {
            #[repr(C)]
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, Hash, PartialEq))]
            #[derive(Copy, Clone)]
            #[allow(deprecated)]
            $(#[$attr])*
            pub struct $i { $($field)* }
        }
    );
}

/// Implement `Clone` and `Copy` for a tuple struct, as well as `Debug`, `Eq`, `Hash`,
/// and `PartialEq` if the `extra_traits` feature is enabled.
///
/// This is the same as [`s`] but works for tuple structs.
macro_rules! s_paren {
    ($(
        $(#[$attr:meta])*
        pub struct $i:ident ( $($field:tt)* );
    )*) => ($(
        __item! {
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, Hash, PartialEq))]
            #[derive(Copy, Clone)]
            $(#[$attr])*
            pub struct $i ( $($field)* );
        }
    )*);
}

/// Implement `Clone` and `Copy` for a struct with no `extra_traits` feature.
///
/// Most items will prefer to use [`s`].
macro_rules! s_no_extra_traits {
    ($(
        $(#[$attr:meta])*
        pub $t:ident $i:ident { $($field:tt)* }
    )*) => ($(
        s_no_extra_traits!(it: $(#[$attr])* pub $t $i { $($field)* });
    )*);

    (it: $(#[$attr:meta])* pub union $i:ident { $($field:tt)* }) => (
        __item! {
            #[repr(C)]
            #[derive(Copy, Clone)]
            $(#[$attr])*
            pub union $i { $($field)* }
        }
    );

    (it: $(#[$attr:meta])* pub struct $i:ident { $($field:tt)* }) => (
        __item! {
            #[repr(C)]
            #[derive(Copy, Clone)]
            $(#[$attr])*
            pub struct $i { $($field)* }
        }
    );
}

/// Specify that an enum should have no traits that aren't specified in the macro
/// invocation, i.e. no `Clone` or `Copy`.
macro_rules! missing {
    ($(
        $(#[$attr:meta])*
        pub enum $i:ident {}
    )*) => ($(
        $(#[$attr])*
        #[allow(missing_copy_implementations)]
        pub enum $i { }
    )*);
}

/// Implement `Clone` and `Copy` for an enum, as well as `Debug`, `Eq`, `Hash`, and
/// `PartialEq` if the `extra_traits` feature is enabled.
macro_rules! e {
    ($(
        $(#[$attr:meta])*
        pub enum $i:ident { $($field:tt)* }
    )*) => ($(
        __item! {
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, Hash, PartialEq))]
            #[derive(Copy, Clone)]
            $(#[$attr])*
            pub enum $i { $($field)* }
        }
    )*);
}

cfg_if! {
    if #[cfg(feature = "const-extern-fn")] {
        /// Define an `unsafe` function that is const as long as `const-extern-fn` is enabled.
        macro_rules! f {
            ($(
                $(#[$attr:meta])*
                pub $({$constness:ident})* fn $i:ident($($arg:ident: $argty:ty),*) -> $ret:ty {
                    $($body:stmt);*
                }
            )*) => ($(
                #[inline]
                $(#[$attr])*
                pub $($constness)* unsafe extern fn $i($($arg: $argty),*) -> $ret {
                    $($body);*
                }
            )*)
        }

        /// Define a safe function that is const as long as `const-extern-fn` is enabled.
        macro_rules! safe_f {
            ($(
                $(#[$attr:meta])*
                pub $({$constness:ident})* fn $i:ident($($arg:ident: $argty:ty),*) -> $ret:ty {
                    $($body:stmt);*
                }
            )*) => ($(
                #[inline]
                $(#[$attr])*
                pub $($constness)* extern fn $i($($arg: $argty),*) -> $ret {
                    $($body);*
                }
            )*)
        }

        /// A nonpublic function that is const as long as `const-extern-fn` is enabled.
        macro_rules! const_fn {
            ($(
                $(#[$attr:meta])*
                $({$constness:ident})* fn $i:ident($($arg:ident: $argty:ty),*) -> $ret:ty {
                    $($body:stmt);*
                }
            )*) => ($(
                #[inline]
                $(#[$attr])*
                $($constness)* fn $i($($arg: $argty),*) -> $ret {
                    $($body);*
                }
            )*)
        }
    } else {
        /// Define an `unsafe` function that is const as long as `const-extern-fn` is enabled.
        macro_rules! f {
            ($(
                $(#[$attr:meta])*
                pub $({$constness:ident})* fn $i:ident($($arg:ident: $argty:ty),*) -> $ret:ty {
                    $($body:stmt);*
                }
            )*) => ($(
                #[inline]
                $(#[$attr])*
                pub unsafe extern fn $i($($arg: $argty),*) -> $ret {
                    $($body);*
                }
            )*)
        }

        /// Define a safe function that is const as long as `const-extern-fn` is enabled.
        macro_rules! safe_f {
            ($(
                $(#[$attr:meta])*
                pub $({$constness:ident})* fn $i:ident($($arg:ident: $argty:ty),*) -> $ret:ty {
                    $($body:stmt);*
                }
            )*) => ($(
                #[inline]
                $(#[$attr])*
                pub extern fn $i($($arg: $argty),*
                ) -> $ret {
                    $($body);*
                }
            )*)
        }

        /// A nonpublic function that is const as long as `const-extern-fn` is enabled.
        macro_rules! const_fn {
            ($(
                $(#[$attr:meta])*
                $({$constness:ident})* fn $i:ident($($arg:ident: $argty:ty),*) -> $ret:ty {
                    $($body:stmt);*
                }
            )*) => ($(
                #[inline]
                $(#[$attr])*
                fn $i($($arg: $argty),*) -> $ret {
                    $($body);*
                }
            )*)
        }
    }
}

macro_rules! __item {
    ($i:item) => {
        $i
    };
}

macro_rules! ptr_addr_of {
    ($place:expr) => {
        ::core::ptr::addr_of!($place)
    };
}
