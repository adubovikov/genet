use genet_derive::Attr;
use genet_sdk::{decoder::*, prelude::*};

struct UdpWorker {
    layer: LayerType<Udp>,
}

impl Worker for UdpWorker {
    fn decode(&mut self, stack: &mut LayerStack) -> Result<Status> {
        let data = stack.top().unwrap().payload();
        let mut layer = Layer::new(&self.layer, &data);
        let payload = data.try_get(self.layer.byte_size()..)?;
        layer.set_payload(&payload);
        stack.add_child(layer);
        Ok(Status::Done)
    }
}

#[derive(Clone)]
struct UdpDecoder {}

impl Decoder for UdpDecoder {
    fn new_worker(&self, _ctx: &Context) -> Box<Worker> {
        Box::new(UdpWorker {
            layer: LayerType::new("udp"),
        })
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            id: "udp".into(),
            ..Metadata::default()
        }
    }
}

#[derive(Attr)]
struct Udp {
    #[genet(alias = "_.src", typ = "@udp:port")]
    src: u16,

    #[genet(alias = "_.dst", typ = "@udp:port")]
    dst: u16,

    length: u16,

    checksum: u16,
}

genet_decoders!(UdpDecoder {});
