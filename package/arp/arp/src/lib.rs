use genet_derive::Attr;
use genet_sdk::{decoder::*, prelude::*};

struct ArpWorker {
    layer: LayerType<ARP>,
}

impl Worker for ArpWorker {
    fn decode(&mut self, stack: &mut LayerStack) -> Result<Status> {
        let data = stack.top().unwrap().payload();
        let mut layer = Layer::new(&self.layer, &data);

        let hwtype = self.layer.hwtype.try_get(&layer)?;
        let protocol = self.layer.protocol.try_get(&layer)?;

        let hlen: usize = self.layer.hlen.try_get(&layer)? as usize;
        let plen: usize = self.layer.plen.try_get(&layer)? as usize;

        let (sha, tha) = match hwtype {
            HardwareType::Eth => (&self.layer.sha_eth, &self.layer.tha_eth),
            _ => return Ok(Status::Skip),
        };

        let (spa, tpa) = match protocol {
            ProtocolType::IPv4 => (&self.layer.spa_ipv4, &self.layer.tpa_ipv4),
            ProtocolType::IPv6 => (&self.layer.spa_ipv6, &self.layer.tpa_ipv6),
            _ => return Ok(Status::Skip),
        };

        let mut offset = self.layer.byte_size();
        layer.add_attr(&sha, offset..offset + hlen);
        offset += hlen;
        layer.add_attr(&spa, offset..offset + plen);
        offset += plen;
        layer.add_attr(&tha, offset..offset + hlen);
        offset += hlen;
        layer.add_attr(&tpa, offset..offset + plen);

        stack.add_child(layer);
        Ok(Status::Done)
    }
}

#[derive(Clone)]
struct ArpDecoder {}

impl Decoder for ArpDecoder {
    fn new_worker(&self, _ctx: &Context) -> Box<Worker> {
        Box::new(ArpWorker {
            layer: LayerType::new("arp"),
        })
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            id: "arp".into(),
            ..Metadata::default()
        }
    }
}

#[derive(Attr)]
struct ARP {
    hwtype: Enum2Field<u16, HardwareType>,
    protocol: Enum2Field<u16, ProtocolType>,
    hlen: Node2Field<u8>,
    plen: Node2Field<u8>,
    op: Enum2Field<u16, OperationType>,

    #[genet(detach, id = "sha", typ = "@eth:mac", alias = "_.src", byte_size = 6)]
    sha_eth: Node2Field<ByteSlice>,

    #[genet(detach, id = "tha", typ = "@eth:mac", alias = "_.dst", byte_size = 6)]
    tha_eth: Node2Field<ByteSlice>,

    #[genet(detach, id = "spa", typ = "@ipv4:addr", byte_size = 4)]
    spa_ipv4: Node2Field<ByteSlice>,

    #[genet(detach, id = "tpa", typ = "@ipv4:addr", byte_size = 4)]
    tpa_ipv4: Node2Field<ByteSlice>,

    #[genet(detach, id = "spa", typ = "@ipv6:addr", byte_size = 16)]
    spa_ipv6: Node2Field<ByteSlice>,

    #[genet(detach, id = "tpa", typ = "@ipv6:addr", byte_size = 16)]
    tpa_ipv6: Node2Field<ByteSlice>,
}

#[derive(Attr)]
enum HardwareType {
    Eth,
    Unknown,
}

impl Default for HardwareType {
    fn default() -> Self {
        HardwareType::Unknown
    }
}

impl From<u16> for HardwareType {
    fn from(data: u16) -> Self {
        match data {
            0x0001 => HardwareType::Eth,
            _ => Self::default(),
        }
    }
}

#[derive(Attr)]
enum ProtocolType {
    IPv4,
    IPv6,
    Unknown,
}

impl Default for ProtocolType {
    fn default() -> Self {
        ProtocolType::Unknown
    }
}

impl From<u16> for ProtocolType {
    fn from(data: u16) -> Self {
        match data {
            0x0800 => ProtocolType::IPv4,
            0x86DD => ProtocolType::IPv6,
            _ => Self::default(),
        }
    }
}

#[derive(Attr)]
enum OperationType {
    Requset,
    Reply,
    Unknown,
}

impl Default for OperationType {
    fn default() -> Self {
        OperationType::Unknown
    }
}

impl From<u16> for OperationType {
    fn from(data: u16) -> Self {
        match data {
            0x0001 => OperationType::Requset,
            0x0002 => OperationType::Reply,
            _ => Self::default(),
        }
    }
}

genet_decoders!(ArpDecoder {});
