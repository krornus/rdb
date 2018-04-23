use std::fs::{File,OpenOptions};
use std::io::{Read,Write,SeekFrom,Seek};
use std::str;
use std::mem::transmute;
use std::ops::Range;

use vm_info::ProcessId;
use vm_info::mapped_region::{self,MemoryRegion,Permissions};
use twoway::find_bytes;

use error::DebugError;

/* FIXME find way to get system wordsize */
/* intel defines a WORD as 16 bits */
const WORDSIZE: usize = 2;


#[derive(PartialEq,Eq)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}

pub enum QuerySize {
    Length,
    Bytes(usize),
    Half,
    Word,
    Double,
    Quad,
}

impl QuerySize {
    fn size(&self, default: usize) -> usize {
        match self {
            &QuerySize::Length => default,
            &QuerySize::Bytes(size) => size,
            &QuerySize::Half => WORDSIZE/2,
            &QuerySize::Word => WORDSIZE,
            &QuerySize::Double => WORDSIZE*2,
            &QuerySize::Quad => WORDSIZE*4,
        }
    }
}

pub trait MemoryPack {
    fn pack(self, size: QuerySize, order: Endianness) -> Vec<u8>;
}

impl MemoryPack for String {
    fn pack(self, size: QuerySize, _: Endianness) -> Vec<u8> {

        let mut bytes: Vec<u8>;
        let qsize = size.size(self.len());
        let pad = qsize as isize - self.len() as isize;

        if pad > 0 {
            bytes = self.into();
            bytes.splice(0..0, vec![0; pad as usize].into_iter());
        } else if pad < 0 {
            let mut s = self.clone();
            s.truncate(qsize);
            bytes = s.into();
        } else {
            bytes = self.into();
        }

        bytes
    }
}

impl<'a> MemoryPack for &'a str {
    fn pack(self, size: QuerySize, _: Endianness) -> Vec<u8> {
        let mut bytes: Vec<u8>;
        let qsize = size.size(self.len());
        let pad = qsize as isize - self.len() as isize;

        if pad > 0 {
            bytes = self.into();
            bytes.splice(0..0, vec![0; pad as usize].into_iter());
        } else if pad < 0 {
            self.to_string().truncate(qsize);
            bytes = self.into();
        } else {
            bytes = self.into();
        }

        bytes
    }
}

macro_rules! impl_transmute_pack {
    ($type:ty, $size:expr) => {
        impl MemoryPack for $type {
            fn pack(self, size: QuerySize, order: Endianness) -> Vec<u8> {

                let mut bytes = unsafe { transmute::<$type, [u8; $size]>(self) }.to_vec();

                let qsize = size.size($size);
                let pad = qsize as isize - bytes.len() as isize;

                if pad > 0 {
                    bytes.extend(vec![0; pad as usize]);
                } else if pad < 0 {
                    bytes.resize(qsize,0);
                }

                if order != Endianness::LittleEndian {
                    bytes.reverse();
                }

                bytes
            }
        }
    }
}

impl_transmute_pack!(i8, 1);
impl_transmute_pack!(u8, 1);

impl_transmute_pack!(i16, 2);
impl_transmute_pack!(u16, 2);

impl_transmute_pack!(i32, 4);
impl_transmute_pack!(u32, 4);

impl_transmute_pack!(i64, 8);
impl_transmute_pack!(u64, 8);

#[cfg(target_pointer_width = "64")]
impl_transmute_pack!(usize, 8);
#[cfg(target_pointer_width = "64")]
impl_transmute_pack!(isize, 8);

#[cfg(target_pointer_width = "32")]
impl_transmute_pack!(usize, 4);
#[cfg(target_pointer_width = "32")]
impl_transmute_pack!(isize, 4);

#[derive(Debug)]
pub struct Memory {
    pub maps: Vec<MemoryRegion>,
    file: File,
}


#[derive(Debug)]
pub struct FoundMemory {
    pub region: MemoryRegion,
    pub offset: usize,
    pub address: usize,
}

#[derive(Debug)]
pub enum ChunkPermission {
    Read,
    Write,
    Execute,
    Shared,
    Private,
}

impl ChunkPermission {
    fn allowed(&self, p: &Permissions) -> bool {
        match self {
            &ChunkPermission::Read => { p.read() },
            &ChunkPermission::Write => { p.write() },
            &ChunkPermission::Execute => { p.execute() },
            &ChunkPermission::Shared => { p.shared() },
            &ChunkPermission::Private => { p.private() },
        }
    }
}

impl Memory {
    pub fn load(pid: usize) -> Result<Memory, DebugError> {

        let path = format!("/proc/{}/mem", pid);

        let memory = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(path)?;

        let mut regions = mapped_region::iter_mappings(ProcessId::Num(pid as u32))?
            .filter_map(|r|{
                r.ok()
            })
            .collect::<Vec<_>>();

        /* no guarantees seen in `man proc` */
        regions.sort_by_key(|r| r.start());

        Ok(Memory {
            file: memory,
            maps: regions,
        })
    }

    pub fn max(&self) -> usize {
        self.maps.last().map_or(0, |r| r.end())
    }

    pub fn min(&self) -> usize {
        self.maps.first().map_or(0, |r| r.start())
    }

    pub fn write(&mut self, addr: usize, data: &[u8]) -> Result<usize, DebugError> {

        if !self.find_chunk(addr, data.len(), ChunkPermission::Write).is_some() {
            return Err("Address is not mapped in memory".into())
        }

        self.file.seek(SeekFrom::Start(addr as u64))?;
        Ok(self.file.write(data)?)

    }

    pub fn read(&mut self, addr: usize, len: usize) -> Result<Vec<u8>, DebugError> {

        if !self.find_chunk(addr, len, ChunkPermission::Read).is_some() {
            return Err("Address is not mapped in memory".into())
        }

        let mut buf = Vec::with_capacity(len);

        self.file.seek(SeekFrom::Start(addr as u64))?; {
            let fref = Write::by_ref(&mut self.file);
            fref.take(len as u64).read_to_end(&mut buf)?;
        }

        Ok(buf)
    }

    pub fn find_chunk(&self, addr: usize, len: usize, permission: ChunkPermission) -> Option<&MemoryRegion> {

        /* coalesce chunks */
        /* don't want to error when two chunks with no gap */
        /* both have desired permission */
        let mut is_start_chunk = true;

        let mut start = 0;
        let mut end = 0;

        let mut map;

        let nmaps = self.maps.len();

        if nmaps == 0 {
            return None
        } else {
            map = &self.maps[0];
        }

        for i in 0..nmaps {

            map = &self.maps[i];
            let permission = permission.allowed(&map.permissions);

            if is_start_chunk && permission {
                start = map.start_address;
                end = map.end_address;
                is_start_chunk = false;
            /* coalesce if next chunk has permission and current end == new start */
            } else if !is_start_chunk && map.start_address == end && permission {
                end = map.end_address;
            /* else, we have a maxed permission chunk */
            } else if !is_start_chunk {
                let range = start..end;

                if permission {
                    start = map.start_address;
                    end = map.end_address;
                } else {
                    is_start_chunk = true;
                }

                if range.contains_value(addr) && addr + len <= range.end() {
                    return Some(map);
                }
            }

            /* post update so that we have initialized value of map if len == 1 */
        }

        let range = start..end;

        if range.contains_value(addr) && addr + len <= range.end() {
            Some(map)
        } else {
            None
        }
    }

    pub fn search<T: AsRef<[u8]>>(&mut self, start: usize, len: usize, value: T)
        -> Vec<FoundMemory>
    {
        let srange = start..start+len;

        self.maps.clone().into_iter().filter(|r| {
            /* r is smaller than start..start+len */
            (r.contains_value(start) || r.contains_value(start+len)) ||
            /* r is larger than start..start+len */
            (srange.contains_value(r.start()) || srange.contains_value(r.end()))
        }).map(|region| {
            let start = region.start();
            let len = region.end()-region.start();
            (region.clone(), self.read(start,len))
        }).filter(|&(_, ref bytes)| {
            bytes.is_ok()
        }).flat_map(|(region, bytes)| {
            Self::get_matches(&region, &value, &bytes.unwrap())
        }).collect()
    }

    fn get_matches<'a, T: AsRef<[u8]>>(region: &'a MemoryRegion, value: T, bytes: &Vec<u8>)
        -> Vec<FoundMemory>
    {
        let mut start = 0;
        let mut regions = vec![];

        while {
            if let Some(found) = find_bytes(&bytes[start..], value.as_ref()) {
                start = found+1;
                regions.push(
                    FoundMemory {
                        region: region.clone(),
                        offset: found,
                        address: region.start_address + found,
                    }
                );
                true
            } else {
                false
            }
        } {}

        regions
    }
}

trait InRange<Idx: PartialOrd + Copy> {
    fn contains_value(&self, value: Idx) -> bool {
        value >= self.start() && value < self.end()
    }
    fn start(&self) -> Idx;
    fn end(&self) -> Idx;
}

impl InRange<usize> for MemoryRegion {
    fn start(&self) -> usize {
        self.start_address
    }

    fn end(&self) -> usize {
        self.end_address
    }
}

impl<T: PartialOrd + Copy> InRange<T> for Range<T> {
    fn start(&self) -> T {
        self.start
    }

    fn end(&self) -> T {
        self.end
    }
}
