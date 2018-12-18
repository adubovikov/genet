use crate::{context::Context, error::Error, layer::LayerStack, result::Result, vec::SafeVec};
use bincode;
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::ptr;

/// Execution type.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ExecType {
    Lazy,
    ParallelSync,
    SerialSync,
}

/// Decoding status.
#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    Done,
    Skip,
}

/// Decoder metadata.
#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exec_type: ExecType,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            exec_type: ExecType::Lazy,
        }
    }
}

/// Decoder worker trait.
pub trait Worker {
    fn decode(&mut self, stack: &mut LayerStack) -> Result<Status>;
}

#[repr(C)]
pub struct WorkerBox {
    decode: extern "C" fn(*mut WorkerBox, *mut LayerStack, *mut Error) -> u8,
    drop: extern "C" fn(*mut Box<Worker>),
    worker: *mut Box<Worker>,
}

impl WorkerBox {
    fn new(worker: Box<Worker>) -> WorkerBox {
        Self {
            decode: abi_decode,
            drop: abi_drop,
            worker: Box::into_raw(Box::new(worker)),
        }
    }

    pub fn decode(&mut self, layer: &mut LayerStack) -> Result<Status> {
        let mut error = Error::new("");
        let result = (self.decode)(self, layer, &mut error);
        match result {
            2 => Ok(Status::Done),
            1 => Ok(Status::Skip),
            _ => Err(Box::new(error)),
        }
    }
}

impl Drop for WorkerBox {
    fn drop(&mut self) {
        (self.drop)(self.worker);
    }
}

extern "C" fn abi_decode(worker: *mut WorkerBox, layer: *mut LayerStack, error: *mut Error) -> u8 {
    let worker = unsafe { &mut *((*worker).worker) };
    let mut layer = unsafe { &mut *layer };
    match worker.decode(&mut layer) {
        Ok(stat) => match stat {
            Status::Done => 2,
            Status::Skip => 1,
        },
        Err(err) => {
            unsafe {
                ptr::write(error, Error::new(err.description()));
            }
            0
        }
    }
}

extern "C" fn abi_drop(worker: *mut Box<Worker>) {
    unsafe { Box::from_raw(worker) };
}

/// Decoder trait.
pub trait Decoder: DecoderClone + Send {
    fn new_worker(&self, ctx: &Context) -> Box<Worker>;
    fn metadata(&self) -> Metadata;
}

pub trait DecoderClone {
    fn clone_box(&self) -> Box<Decoder>;
}

impl<T> DecoderClone for T
where
    T: 'static + Decoder + Clone,
{
    fn clone_box(&self) -> Box<Decoder> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Decoder> {
    fn clone(&self) -> Box<Decoder> {
        self.clone_box()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DecoderBox {
    new_worker: extern "C" fn(*const DecoderBox, *const Context) -> WorkerBox,
    metadata: extern "C" fn(*const DecoderBox) -> SafeVec<u8>,
    decoder: *mut Box<Decoder>,
}

unsafe impl Send for DecoderBox {}

impl DecoderBox {
    pub fn new<T: 'static + Decoder>(diss: T) -> DecoderBox {
        let diss: Box<Decoder> = Box::new(diss);
        Self {
            new_worker: abi_new_worker,
            metadata: abi_metadata,
            decoder: Box::into_raw(Box::new(diss)),
        }
    }

    pub fn new_worker(&self, ctx: &Context) -> WorkerBox {
        (self.new_worker)(self, ctx)
    }

    pub fn metadata(&self) -> Metadata {
        bincode::deserialize(&(self.metadata)(self)).unwrap()
    }
}

impl Serialize for DecoderBox {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.metadata().serialize(serializer)
    }
}

extern "C" fn abi_new_worker(diss: *const DecoderBox, ctx: *const Context) -> WorkerBox {
    let diss = unsafe { &*(*diss).decoder };
    let ctx = unsafe { &(*ctx) };
    WorkerBox::new(diss.new_worker(ctx))
}

extern "C" fn abi_metadata(diss: *const DecoderBox) -> SafeVec<u8> {
    let diss = unsafe { &*((*diss).decoder) };
    bincode::serialize(&diss.metadata()).unwrap().into()
}

#[cfg(test)]
mod tests {
    use crate::{
        attr::AttrClass,
        context::Context,
        decoder::{Decoder, DecoderBox, ExecType, Metadata, Status, Worker},
        fixed::Fixed,
        layer::{Layer, LayerClass, LayerStack, LayerStackData},
        result::Result,
        slice::ByteSlice,
        token::Token,
    };

    #[test]
    fn decode() {
        struct TestWorker {}

        impl Worker for TestWorker {
            fn decode(&mut self, stack: &mut LayerStack) -> Result<Status> {
                let attr = Fixed::new(AttrClass::builder(Token::from(1234)).build());
                let class = Fixed::new(LayerClass::builder(attr).build());
                let layer = Layer::new(class, ByteSlice::new());
                stack.add_child(layer);
                Ok(Status::Done)
            }
        }

        #[derive(Clone)]
        struct TestDecoder {}

        impl Decoder for TestDecoder {
            fn new_worker(&self, _ctx: &Context) -> Box<Worker> {
                Box::new(TestWorker {})
            }

            fn metadata(&self) -> Metadata {
                Metadata {
                    exec_type: ExecType::ParallelSync,
                    ..Metadata::default()
                }
            }
        }

        let ctx = Context::default();
        let diss = DecoderBox::new(TestDecoder {});
        let mut worker = diss.new_worker(&ctx);

        let attr = Fixed::new(AttrClass::builder(Token::null()).build());
        let class = Fixed::new(LayerClass::builder(attr).build());
        let mut layer = Layer::new(class, ByteSlice::new());
        let mut data = LayerStackData {
            children: Vec::new(),
        };
        let mut layer = LayerStack::from_mut_ref(&mut data, &mut layer);

        assert_eq!(worker.decode(&mut layer).unwrap(), Status::Done);
    }
}
