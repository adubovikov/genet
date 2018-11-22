//! Helper macros.

/// Creates a LayerClass.
#[macro_export]
macro_rules! layer_class {
    ($header:expr) => (::genet_sdk::layer::LayerClass::builder($header).build());
    ($header:expr, $($key:ident : $($arg:expr)*),*) => (::genet_sdk::layer::LayerClass::builder($header)
                $( . $key ( $($arg),* ) )*
                .build()
    );
}

/// Creates a LayerClass lazily.
#[macro_export]
macro_rules! layer_class_lazy {
    () => (
        {
            lazy_static! {
                static ref LAYER_CLASS : genet_sdk::layer::LayerClass = layer_class!();
            }
            &LAYER_CLASS
        }
    );
    ($($key:ident : $($arg:expr)*),*) => (
        {
            lazy_static! {
                static ref LAYER_CLASS : genet_sdk::layer::LayerClass = layer_class!($($key : $($arg)* ),* );
            }
            &LAYER_CLASS
        }
    );
}

/// Defines a LayerClass.
#[macro_export]
macro_rules! def_layer_class {
    ($name:ident, $header:expr) => (
        lazy_static! {
            static ref $name : genet_sdk::layer::LayerClass = layer_class!($header);
        }
    );
    ($name:ident, $header:expr, $($key:ident : $($arg:expr)*),*) => (
        lazy_static! {
            static ref $name : genet_sdk::layer::LayerClass = layer_class!($header, $($key : $($arg)* ),* );
        }
    );
}

/// Creates an AttrClass.
#[macro_export]
macro_rules! attr_class {
    ($id:expr) => (::genet_sdk::attr::AttrClass::builder($id).build());
    ($id:expr, $($key:ident : $($arg:expr)*),*) => (::genet_sdk::attr::AttrClass::builder($id)
                $( . $key ( $($arg),* ) )*
                .build());
}

/// Creates an AttrClass lazily.
#[macro_export]
macro_rules! attr_class_lazy {
    ($id:expr) => (
        {
            lazy_static! {
                static ref ATTR_CLASS : genet_sdk::attr::AttrClass = attr_class!($id);
            }
            &ATTR_CLASS
        }
    );
    ($id:expr, $($key:ident : $($arg:expr)*),*) => (
        {
            lazy_static! {
                static ref ATTR_CLASS : genet_sdk::attr::AttrClass = attr_class!($id, $($key : $($arg)* ),* );
            }
            &ATTR_CLASS
        }
    );
}

/// Defines an AttrClass.
#[macro_export]
macro_rules! def_attr_class {
    ($name:ident, $id:expr) => (
        lazy_static! {
            static ref $name : genet_sdk::attr::AttrClass = attr_class!($id);
        }
    );
    ($name:ident, $id:expr, $($key:ident : $($arg:expr)*),*) => (
        lazy_static! {
            static ref $name : genet_sdk::attr::AttrClass = attr_class!($id, $($key : $($arg)* ),* );
        }
    );
}
