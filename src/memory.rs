use std::fs::File;
use std::io::{Read,SeekFrom,Seek};
use std::str;
use std::ops::Range;

use regex::bytes;
use vm_info::ProcessId;
use vm_info::mapped_region::{self,MemoryRegion};

use error::DebugError;

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

impl Memory {
    pub fn load(pid: usize) -> Result<Memory, DebugError> {

        let path = format!("/proc/{}/mem", pid);

        let memory = File::open(path)?;
        let regions = mapped_region::iter_mappings(ProcessId::Num(pid as u32))?
            .filter_map(|r|{
                r.ok()
            })
            .collect::<Vec<_>>();

        Ok(Memory {
            file: memory,
            maps: regions,
        })
    }

    pub fn max(&self) -> usize {
        self.maps.iter().map(|r| r.end()).max().unwrap_or(0)
    }

    pub fn min(&self) -> usize {
        self.maps.iter().map(|r| r.end()).min().unwrap_or(0)
    }

    pub fn read(&mut self, addr: usize, len: usize) -> Result<Vec<u8>, DebugError> {

        self.maps.iter().find(|m| {
            (m.start_address..m.end_address).contains_value(addr) &&
            addr + len <= m.end_address
        }).ok_or(DebugError::Error("Address is not mapped in memory"))?;

        let mut buf = Vec::with_capacity(len);

        self.file.seek(SeekFrom::Start(addr as u64))?; {
            let fref = self.file.by_ref();
            fref.take(len as u64).read_to_end(&mut buf)?;
        }

        Ok(buf)
    }

    /* TODO make search with regex possible */
    /* TODO need a way to differentiate results */
    pub fn search<T: AsRef<[u8]>>(&mut self, start: usize, len: usize, values: Vec<T>, size: usize)
        -> Vec<FoundMemory>
    {

        let rexprs = values.iter().filter_map(|v| regex_from_u8(v.as_ref(), size).ok()).collect();
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
            Self::get_matches(&region, &rexprs, &bytes.unwrap())
        }).collect()
    }

    /* FIXME remove collect */
    fn get_matches<'a>(region: &'a MemoryRegion, rexprs: &'a Vec<bytes::Regex>, bytes: &[u8])
        -> Vec<FoundMemory>
    {
            rexprs.iter().flat_map(|re| {
                re.find_iter(bytes).map(|m| {
                    FoundMemory {
                        region: region.clone(),
                        offset: m.start(),
                        address: region.start_address + m.start(),
                    }
                })
            }).collect()
    }
}

/* TODO dont loop regex, add a union operator instead */
fn regex_from_u8(bytes: &[u8], len: usize) -> Result<bytes::Regex, DebugError> {
    let pad = bytes.len() as isize - len as isize;

    if pad < 0 {
        return Err(DebugError::Error("More search bytes than search size"));
    }

    let pvec = Vec::with_capacity(pad as usize);
    let s = format!(r"(?-u){}{}", str::from_utf8(&pvec[0..pad as usize])?, str::from_utf8(bytes)?);

    Ok(bytes::Regex::new(&s)?)
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
