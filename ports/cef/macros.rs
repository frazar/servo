/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![macro_escape]

// Provides the implementation of a CEF class. An example follows:
//
//    struct ServoCefThing {
//        ...
//    }
//
//    cef_class_impl! {
//        ServoCefThing : CefThing, cef_thing_t {
//            // Declare method implementations using the *C* API. (This may change later, once we
//            // have associated types in Rust.)
//            //
//            // Note that if the method returns unit, you must write `-> ()` explicitly. This is
//            // due to limitations of Rust's macro system.
//            fn foo(&this, a: int, b: *mut cef_other_thing_t) -> () {
//                // Inside here, `a` will have type `int`, and `b` will have the type
//                // `CefOtherThing` -- i.e. the Rust-wrapped version of `cef_other_thing_t`.
//                ...
//            }
//
//            fn bar(&this, a: int) -> *mut cef_other_thing_t {
//                // Return types are automatically unwrapped from the Rust types (e.g.
//                // `CefOtherThing`) into the corresponding C types (e.g. `*mut
//                // cef_other_thing_t`).
//                let x: CefOtherThing = ...;
//                x
//            }
//        }
//    }
macro_rules! cef_class_impl(
    ($class_name:ident : $interface_name:ident, $c_interface_name:ident {
        $(
            fn $method_name:ident ( & $method_this:ident
                                   $( , $method_arg_name:ident : $method_arg_type:ty )* )
                                   -> $method_return_type:ty $method_body:block
        )*
    }) => (
        impl $class_name {
            pub fn as_cef_interface(self) -> $interface_name {
                let cef_object = unsafe {
                    $interface_name::from_c_object_addref(
                        ::eutil::create_cef_object::<$c_interface_name,$class_name>())
                };
                unsafe {
                    $((*cef_object.c_object()).$method_name = Some($method_name);)*
                    let extra_slot =
                        ::std::mem::transmute::<&mut u8,
                                                &mut $class_name>(&mut (*cef_object.c_object())
                                                                                   .extra);
                    ::std::ptr::write(extra_slot, self);
                }
                cef_object
            }
        }

        $(
            extern "C" fn $method_name(raw_this: *mut $c_interface_name,
                                       $($method_arg_name: $method_arg_type),*)
                                       -> $method_return_type {
                let $method_this = unsafe {
                    $interface_name::from_c_object_addref(raw_this)
                };
                $(
                    let $method_arg_name = unsafe {
                        ::wrappers::CefWrap::to_rust($method_arg_name)
                    };
                )*
                ::wrappers::CefWrap::to_c($method_body)
            }
        )*

        impl ::eutil::Downcast<$class_name> for $interface_name {
            fn downcast(&self) -> &$class_name {
                unsafe {
                    ::std::mem::transmute::<&u8,&$class_name>(&(*self.c_object()).extra)
                }
            }
        }
    )
)

macro_rules! cef_static_method_impls(
    (
        $(
            fn $method_name:ident ( $($method_arg_name:ident : $method_arg_type:ty ),* )
                                   -> $method_return_type:ty $method_body:block
        )*
    ) => (
        $(
            pub extern "C" fn $method_name($($method_arg_name: $method_arg_type),*)
                                           -> $method_return_type {
                $(
                    let $method_arg_name = unsafe {
                        ::wrappers::CefWrap::to_rust($method_arg_name)
                    };
                )*
                ::wrappers::CefWrap::to_c($method_body)
            }
        )*
    )
)

macro_rules! cef_stub_static_method_impls(
    (
        $(
            fn $method_name:ident ( $($method_arg_name:ident : $method_arg_type:ty ),* )
                                   -> $method_return_type:ty ;
        )*
    ) => (
        $(
            pub extern "C" fn $method_name($(_: $method_arg_type),*)
                                           -> $method_return_type {
                panic!("unimplemented static method: {}", stringify!($method_name))
            }
        )*
    )
)
