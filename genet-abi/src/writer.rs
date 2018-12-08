use crate::{
    context::Context,
    error::Error,
    file::FileType,
    fixed::MutFixed,
    layer::{Layer, LayerStack},
    result::Result,
    vec::SafeVec,
};
use bincode;
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::{fmt, mem, ptr, slice, str};

/// Writer metadata.
#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filters: Vec<FileType>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            filters: Vec::new(),
        }
    }
}

/// Writer trait.
pub trait Writer: Send {
    fn new_worker(&self, ctx: &Context, args: &str) -> Result<Box<Worker>>;
    fn metadata(&self) -> Metadata;
}

type WriterNewWorkerFunc = extern "C" fn(
    *mut Box<Writer>,
    *const Context,
    *const u8,
    u64,
    *mut WorkerBox,
    *mut Error,
) -> u8;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WriterBox {
    writer: *mut Box<Writer>,
    new_worker: WriterNewWorkerFunc,
    metadata: extern "C" fn(*const WriterBox) -> SafeVec<u8>,
}

unsafe impl Send for WriterBox {}

impl WriterBox {
    pub fn new<T: 'static + Writer>(writer: T) -> WriterBox {
        let writer: Box<Writer> = Box::new(writer);
        Self {
            writer: Box::into_raw(Box::new(writer)),
            new_worker: abi_writer_new_worker,
            metadata: abi_metadata,
        }
    }

    pub fn new_worker(&self, ctx: &Context, args: &str) -> Result<WorkerBox> {
        let mut out: WorkerBox = unsafe { mem::uninitialized() };
        let mut err = Error::new("");
        if (self.new_worker)(
            self.writer,
            ctx,
            args.as_ptr(),
            args.len() as u64,
            &mut out,
            &mut err,
        ) == 1
        {
            Ok(out)
        } else {
            mem::forget(out);
            Err(Box::new(err))
        }
    }

    pub fn metadata(&self) -> Metadata {
        bincode::deserialize(&(self.metadata)(self)).unwrap()
    }
}

impl Serialize for WriterBox {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.metadata().serialize(serializer)
    }
}

extern "C" fn abi_writer_new_worker(
    writer: *mut Box<Writer>,
    ctx: *const Context,
    arg: *const u8,
    arg_len: u64,
    out: *mut WorkerBox,
    err: *mut Error,
) -> u8 {
    let writer = unsafe { &*writer };
    let ctx = unsafe { &*ctx };
    let arg = unsafe { str::from_utf8_unchecked(slice::from_raw_parts(arg, arg_len as usize)) };
    match writer.new_worker(ctx, arg) {
        Ok(worker) => {
            unsafe { ptr::write(out, WorkerBox::new(worker)) };
            1
        }
        Err(e) => {
            unsafe { *err = Error::new(e.description()) };
            0
        }
    }
}

extern "C" fn abi_metadata(writer: *const WriterBox) -> SafeVec<u8> {
    let writer = unsafe { &*((*writer).writer) };
    bincode::serialize(&writer.metadata()).unwrap().into()
}

/// Writer worker trait.
pub trait Worker: Send {
    fn write(&mut self, index: u32, stack: &LayerStack) -> Result<()>;
    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

type WriterFunc = extern "C" fn(*mut Box<Worker>, u32, *const *const Layer, u64, *mut Error) -> u8;

type WriterEndFunc = extern "C" fn(*mut Box<Worker>, *mut Error) -> u8;

pub struct WorkerBox {
    worker: *mut Box<Worker>,
    write: WriterFunc,
    end: WriterEndFunc,
    drop: extern "C" fn(*mut Box<Worker>),
}

unsafe impl Send for WorkerBox {}

impl WorkerBox {
    pub fn new(worker: Box<Worker>) -> WorkerBox {
        Self {
            worker: Box::into_raw(Box::new(worker)),
            write: abi_writer_worker_write,
            end: abi_writer_worker_end,
            drop: abi_writer_worker_drop,
        }
    }

    pub fn write(&mut self, index: u32, layers: &[MutFixed<Layer>]) -> Result<()> {
        let mut e = Error::new("");
        let stack = layers.as_ptr() as *const *const Layer;
        if (self.write)(self.worker, index, stack, layers.len() as u64, &mut e) == 0 {
            Err(Box::new(e))
        } else {
            Ok(())
        }
    }

    pub fn end(&mut self) -> Result<()> {
        let mut e = Error::new("");
        if (self.end)(self.worker, &mut e) == 0 {
            Err(Box::new(e))
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for WorkerBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WorkerBox")
    }
}

impl Drop for WorkerBox {
    fn drop(&mut self) {
        (self.drop)(self.worker);
    }
}

extern "C" fn abi_writer_worker_drop(worker: *mut Box<Worker>) {
    unsafe { Box::from_raw(worker) };
}

extern "C" fn abi_writer_worker_write(
    worker: *mut Box<Worker>,
    index: u32,
    layers: *const *const Layer,
    len: u64,
    err: *mut Error,
) -> u8 {
    let worker = unsafe { &mut *worker };
    let stack = unsafe { LayerStack::new(layers, len as usize) };
    match worker.write(index, &stack) {
        Ok(()) => 1,
        Err(e) => {
            unsafe { *err = Error::new(e.description()) };
            0
        }
    }
}

extern "C" fn abi_writer_worker_end(worker: *mut Box<Worker>, err: *mut Error) -> u8 {
    let worker = unsafe { &mut *worker };
    match worker.end() {
        Ok(()) => 1,
        Err(e) => {
            unsafe { *err = Error::new(e.description()) };
            0
        }
    }
}
