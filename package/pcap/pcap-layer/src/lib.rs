use genet_derive::Attr;
use genet_sdk::{cast, decoder::*, prelude::*};

struct PcapLayerWorker {
    layer: LayerType<Link>,
    eth: WorkerBox,
}

impl Worker for PcapLayerWorker {
    fn decode(&mut self, stack: &mut LayerStack, _data: &ByteSlice) -> Result<Status> {
        let data;

        if stack.id() == token!("[pcap]") {
            data = stack.data();
        } else {
            return Ok(Status::Skip);
        }

        let layer = Layer::new(self.layer.as_ref().clone(), &data);
        stack.add_child(layer);

        let payload = data.try_get(self.layer.byte_size()..)?;
        self.eth.decode(stack, &payload)
    }
}

#[derive(Clone)]
struct PcapLayerDecoder {}

impl Decoder for PcapLayerDecoder {
    fn new_worker(&self, ctx: &Context) -> Box<Worker> {
        Box::new(PcapLayerWorker {
            layer: LayerType::new("link", Link::default()),
            eth: ctx.decoder("eth").unwrap(),
        })
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            id: "pcap_layer".into(),
            exec_type: ExecType::ParallelSync,
            ..Metadata::default()
        }
    }
}

#[derive(Attr, Default)]
struct Link {
    r#type: cast::UInt32BE,
    payload_length: cast::UInt32BE,
    original_length: cast::UInt32BE,

    #[genet(
        typ = "@datetime:unix",
        map = "(x >> 32) as f64 + (x & 0xffff_ffff) as f64 / 1_000_000f64"
    )]
    timestamp: Node<cast::UInt64BE, Timestamp>,
}

#[derive(Attr, Default)]
struct Timestamp {
    sec: cast::UInt32BE,
    usec: cast::UInt32BE,
}

genet_decoders!(PcapLayerDecoder {});
