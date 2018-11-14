extern crate bincode;
extern crate genet_format;
extern crate genet_sdk;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use genet_sdk::{prelude::*, writer::*};

use std::{
    fs::File,
    io::{BufWriter, Write},
};

#[derive(Deserialize)]
struct Arg {
    file: String,
}

#[derive(Clone)]
struct GenetFileWriter {}

impl Writer for GenetFileWriter {
    fn new_worker(&self, _ctx: &Context, arg: &str) -> Result<Box<Worker>> {
        let arg: Arg = serde_json::from_str(arg)?;
        let file = File::create(&arg.file)?;
        let writer = BufWriter::new(file);
        Ok(Box::new(GenetFileWorker {
            writer,
            tokens: Vec::new(),
            attrs: Vec::new(),
            entries: Vec::new(),
        }))
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            id: "app.genet.writer.genet-file".into(),
            filters: vec![FileType::new("genet", &["genet"])],
            ..Metadata::default()
        }
    }
}

struct GenetFileWorker {
    writer: BufWriter<File>,
    tokens: Vec<Token>,
    attrs: Vec<(Token, Token)>,
    entries: Vec<genet_format::Entry>,
}

impl GenetFileWorker {
    fn get_token_index(&mut self, id: Token) -> usize {
        if let Some(index) = self.tokens.iter().position(|x| *x == id) {
            return index;
        }
        self.tokens.push(id);
        self.tokens.len() - 1
    }

    fn get_attr_index(&mut self, id: Token, typ: Token) -> usize {
        if let Some(index) = self.attrs.iter().position(|x| *x == (id, typ)) {
            return index;
        }
        self.attrs.push((id, typ));
        self.attrs.len() - 1
    }
}

impl Worker for GenetFileWorker {
    fn write(&mut self, _index: u32, stack: &LayerStack) -> Result<()> {
        if let Some(layer) = stack.bottom() {
            let mut attrs = Vec::new();
            let id = self.get_token_index(layer.id());
            self.entries.push(genet_format::Entry {
                frame: genet_format::Frame {
                    id,
                    len: layer.data().len(),
                    attrs,
                },
                data: layer.data(),
            });
        }
        Ok(())
    }

    fn end(&mut self) -> Result<()> {
        let attrs = self
            .attrs
            .clone()
            .iter()
            .map(|(id, typ)| genet_format::AttrClass {
                id: self.get_token_index(*id),
                typ: self.get_token_index(*typ),
            })
            .collect();
        let header = genet_format::Header {
            tokens: self.tokens.iter().map(|x| x.to_string()).collect(),
            attrs,
            entries: self.entries.len(),
        };
        let bin = bincode::serialize(&header)?;
        self.writer.write_all(&bincode::serialize(&bin.len())?)?;
        self.writer.write_all(&bin)?;
        for e in self.entries.iter() {
            let bin = bincode::serialize(&e.frame)?;
            self.writer.write_all(&bincode::serialize(&bin.len())?)?;
            self.writer.write_all(&bin)?;
            self.writer.write_all(&e.data)?;
        }
        Ok(())
    }
}

genet_writers!(GenetFileWriter {});
