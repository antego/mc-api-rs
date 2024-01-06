use std::{ffi::CStr, os::fd::AsRawFd, os::raw::c_char, path::Path};

use media_ffi::{
    media_device_info, media_v2_entity, media_v2_interface, media_v2_link, media_v2_pad,
};
use nix::errno::Errno;

mod media_ffi;

nix::ioctl_readwrite!(
    media_ioc_device_info,
    b'|',
    0x00,
    media_ffi::media_device_info
);
nix::ioctl_readwrite!(
    media_ioc_g_topology,
    b'|',
    0x04,
    media_ffi::media_v2_topology
);

#[derive(Debug)]
pub struct MediaDeviceInfo {
    pub driver: String,
    pub model: String,
    pub serial: String,
    pub bus_info: String,
    pub media_version: u32,
    pub hw_version: u32,
    pub driver_version: u32,
}

impl MediaDeviceInfo {
    fn from_ffi(info: &media_device_info) -> MediaDeviceInfo {
        MediaDeviceInfo {
            driver: c_str_to_str(&info.driver),
            model: c_str_to_str(&info.model),
            serial: c_str_to_str(&info.serial),
            bus_info: c_str_to_str(&info.bus_info),
            media_version: info.media_version,
            hw_version: info.hw_revision,
            driver_version: info.driver_version,
        }
    }
}

#[derive(Debug)]
pub struct MediaV2Entity {
    pub id: u32,
    pub name: String,
    pub function: u32,
    pub flags: u32,
}

impl MediaV2Entity {
    fn from_ffi(entity: &media_v2_entity) -> MediaV2Entity {
        MediaV2Entity {
            name: c_str_to_str(&entity.name),
            id: entity.id,
            flags: entity.flags,
            function: entity.function,
        }
    }
}

#[derive(Debug)]
pub struct MediaV2IntfDevnode {
    pub major: u32,
    pub minor: u32,
}

#[derive(Debug)]
pub struct MediaV2Interface {
    pub id: u32,
    pub intf_type: u32,
    pub flags: u32,
    // todo devnode
}

impl MediaV2Interface {
    fn from_ffi(intf: &media_v2_interface) -> MediaV2Interface {
        MediaV2Interface {
            id: intf.id,
            flags: intf.flags,
            intf_type: intf.intf_type,
        }
    }
}

#[derive(Debug)]
pub struct MediaV2Pad {
    pub id: u32,
    pub entity_id: u32,
    pub flags: u32,
    pub index: u32,
}

impl MediaV2Pad {
    fn from_ffi(pad: &media_v2_pad) -> MediaV2Pad {
        MediaV2Pad {
            id: pad.id,
            entity_id: pad.entity_id,
            flags: pad.flags,
            index: pad.index,
        }
    }
}

#[derive(Debug)]
pub struct MediaV2Link {
    pub id: u32,
    pub source_id: u32,
    pub sink_id: u32,
    pub flags: u32,
}

impl MediaV2Link {
    fn from_ffi(pad: &media_v2_link) -> MediaV2Link {
        MediaV2Link {
            id: pad.id,
            source_id: pad.source_id,
            sink_id: pad.sink_id,
            flags: pad.flags,
        }
    }
}

#[derive(Debug)]
pub struct MediaV2Topology {
    pub topology_version: u64,
    pub entities: Vec<MediaV2Entity>,
    pub interfaces: Vec<MediaV2Interface>,
    pub pads: Vec<MediaV2Pad>,
    pub links: Vec<MediaV2Link>,
}

pub fn get_device_info(path: &Path) -> Result<MediaDeviceInfo, Errno> {
    let video_device = std::fs::File::open(path).unwrap();
    let video_device = video_device.as_raw_fd();
    let mut dev_info: media_ffi::media_device_info = unsafe { std::mem::zeroed() };

    let result = unsafe { media_ioc_device_info(video_device, &mut dev_info) };
    match result {
        Ok(_) => return Result::Ok(MediaDeviceInfo::from_ffi(&dev_info)),
        Err(err) => return Result::Err(err),
    }
}

#[derive(Debug)]
pub enum GetTopologyError {
    IoctlError(Errno),
    VersionChange { old_version: u64, new_version: u64 },
}

pub fn get_topology(path: &Path) -> Result<MediaV2Topology, GetTopologyError> {
    let video_device = std::fs::File::open(path).unwrap();
    let video_device = video_device.as_raw_fd();
    let mut topology: media_ffi::media_v2_topology = unsafe { std::mem::zeroed() };

    let res = unsafe { media_ioc_g_topology(video_device, &mut topology) };

    match res {
        Err(err) => return Result::Err(GetTopologyError::IoctlError(err)),
        Ok(_) => (),
    }

    let version = topology.topology_version;

    let mut entities: Vec<media_v2_entity> =
        Vec::with_capacity(topology.num_entities.try_into().unwrap());
    let mut interfaces: Vec<media_v2_interface> =
        Vec::with_capacity(topology.num_interfaces.try_into().unwrap());
    let mut pads: Vec<media_v2_pad> = Vec::with_capacity(topology.num_pads.try_into().unwrap());
    let mut links: Vec<media_v2_link> = Vec::with_capacity(topology.num_links.try_into().unwrap());

    unsafe {
        topology.ptr_entities = entities.as_mut_ptr() as u64;
        topology.ptr_interfaces = interfaces.as_mut_ptr() as u64;
        topology.ptr_pads = pads.as_mut_ptr() as u64;
        topology.ptr_links = links.as_mut_ptr() as u64;
        let res = media_ioc_g_topology(video_device, &mut topology);
        if let Err(errno) = res {
            return Result::Err(GetTopologyError::IoctlError(errno));
        }
        if topology.topology_version != version {
            return Result::Err(GetTopologyError::VersionChange {
                old_version: version,
                new_version: topology.topology_version,
            });
        }
        entities.set_len(topology.num_entities.try_into().unwrap());
        interfaces.set_len(topology.num_interfaces.try_into().unwrap());
        pads.set_len(topology.num_pads.try_into().unwrap());
        links.set_len(topology.num_links.try_into().unwrap());
    };

    let entities: Vec<MediaV2Entity> = entities
        .iter()
        .map(|e| MediaV2Entity::from_ffi(e))
        .collect();

    let interfaces: Vec<MediaV2Interface> = interfaces
        .iter()
        .map(|i| MediaV2Interface::from_ffi(i))
        .collect();

    let pads: Vec<MediaV2Pad> = pads.iter().map(|i| MediaV2Pad::from_ffi(i)).collect();

    let links: Vec<MediaV2Link> = links.iter().map(|i| MediaV2Link::from_ffi(i)).collect();

    let topology = MediaV2Topology {
        topology_version: topology.topology_version,
        entities,
        interfaces,
        pads,
        links,
    };

    return Result::Ok(topology);
}

fn c_str_to_str(c_str: &[c_char]) -> String {
    CStr::from_bytes_until_nul(c_str)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
