use genet_derive::Attr;
use genet_sdk::{decoder::*, prelude::*};

struct IPv6Worker {
    layer: LayerType<IPv6>,
    tcp: DecoderStack,
    udp: DecoderStack,
}

impl Worker for IPv6Worker {
    fn decode(&mut self, stack: &mut LayerStack) -> Result<Status> {
        let data = stack.top().unwrap().payload();
        let mut layer = Layer::new(&self.layer, &data);

        let nheader = self.layer.next_header.try_get(&layer)?;
        loop {
            match nheader {
                // TODO:
                // case 0 | 60 # Hop-by-Hop Options, Destination Options
                // case 43  # Routing
                // case 44  # Fragment
                // case 51  # Authentication Header
                // case 50  # Encapsulating Security Payload
                // case 135 # Mobility
                // No Next Header
                59 => break,
                _ => break,
            }
        }

        let range = self.layer.next_header.byte_range();
        let typ = self.layer.protocol.try_get_range(&layer, range.clone());
        layer.add_attr(&self.layer.protocol, range.clone());

        let payload = data.try_get(self.layer.byte_size()..)?;
        layer.set_payload(&payload);
        stack.add_child(layer);

        match typ {
            Ok(ProtoType::TCP) => self.tcp.decode(stack),
            Ok(ProtoType::UDP) => self.udp.decode(stack),
            _ => Ok(Status::Done),
        }
    }
}

#[derive(Clone)]
struct IPv6Decoder {}

impl Decoder for IPv6Decoder {
    fn new_worker(&self, ctx: &Context) -> Box<Worker> {
        Box::new(IPv6Worker {
            layer: LayerType::new("ipv6"),
            tcp: ctx.decoder("tcp").unwrap(),
            udp: ctx.decoder("udp").unwrap(),
        })
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            id: "ipv6".into(),
            ..Metadata::default()
        }
    }
}

#[derive(Attr)]
struct IPv6 {
    #[attr(bit_size = 4, map = "x >> 4")]
    version: u8,

    #[attr(bit_size = 8, map = "(x >> 4) & 0xff")]
    traffic_class: u16,

    #[attr(
        bit_size = 20,
        map = "(((x[2] as u32) & 0xf) << 16) | ((x[1] as u32) << 8) | x[2] as u32"
    )]
    flow_label: Cast<ByteSlice, u32>,

    payload_length: u16,

    next_header: Node<u8>,

    hop_limit: u8,

    #[attr(alias = "_.dst", typ = "@ipv6:addr", byte_size = 16)]
    dst: ByteSlice,

    #[attr(alias = "_.src", typ = "@ipv6:addr", byte_size = 16)]
    src: ByteSlice,

    #[attr(detach)]
    protocol: Enum<u8, ProtoType>,
}

#[derive(Attr)]
enum ProtoType {
    ICPM,
    IGMP,
    TCP,
    UDP,
    Unknown,
}

impl Default for ProtoType {
    fn default() -> Self {
        ProtoType::Unknown
    }
}

impl From<u8> for ProtoType {
    fn from(data: u8) -> Self {
        match data {
            0x3a => ProtoType::ICPM,
            0x02 => ProtoType::IGMP,
            0x06 => ProtoType::TCP,
            0x11 => ProtoType::UDP,
            _ => Self::default(),
        }
    }
}

genet_decoders!(IPv6Decoder {});
