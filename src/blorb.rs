use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};

/// The usage information for an `IndexEntry`.
#[derive(Debug)]
pub enum Usage {
    /// Indentifier: 'Pict'.
    /// Indicates the resource is an image.
    Pict,

    /// Indentifier: 'Snd '.
    /// Indicates the resource is a sound.
    Snd,

    /// Indentifier: 'Data'.
    /// Indicates the resource is some data.
    Data,

    /// Indentifier: 'Exec'.
    /// Indicates the resource is an executable.
    Exec,
}


/// Contains the usage information for an entry, the resource number of
/// the entry, and where in the blob the entry starts.
#[derive(Debug)]
pub struct IndexEntry {
    pub usage: Usage,
    pub num: u32,
    start: u32,
}


/// Container for list of resource index entries.
#[derive(Debug)]
pub struct ResourceIndex {
    pub entries: HashMap<usize, IndexEntry>,
}


/// Container for chunk metadata. Used for identifying a chunk without
/// loading the full chunk into memory.
#[derive(Debug)]
pub struct ChunkData {
    pub id: String,
    len: u32,
}

/// Representation for loaded blorb chunks
pub enum Chunk {

    /// Chunk returned when the loaded chunk type is unable to be
    /// identified. Per Specification, the machine must ignore unknown
    /// chunks, and this type will be used to do so when necessary.
    Unknown{meta: ChunkData, data: Vec<u8>},

    /// Identifier: 'RIdx'.
    /// Contains a resource index for the IF.
    /// This chunk is mandatory and must be the first chunk in the form.
    ResourceIndex{index: ResourceIndex},

    /// Identifier: 'IFmd',
    /// Contains xml metadata content for the IF.
    Metadata{info: String},

    /// Identifier: 'Fspec',
    /// Contains a reference to a frontispiece image.
    /// This chunk is optional.
    Frontispiece{num: u32},

    /// Identifier: 'GLUL',
    /// Contains Glulx executable
    /// This is a executable resource chunk.
    Glulx{code: Vec<u8>},

    /// Identifier: 'PNG ',
    /// Contains a PNG image
    /// This is a picture resource chunk.
    Png{data: Vec<u8>},

    /// Identifier: 'JPEG',
    /// Contains a JPEG image
    /// This is a picture resource chunk.
    Jpeg{data: Vec<u8>},
}

pub struct Blorb<R: Read + Seek> {
    pub len: u32,
    index: ResourceIndex,
    file: R,
}

impl<R: Read + Seek> Blorb<R> {

    /// Creates a new Blorb from a file. The code goes through the
    /// given game file, validates the file type, and extracts the
    /// basic game objects for the blorb.
    pub fn from_file(file: R) -> Result<Blorb<R>, Error> {
        let mut file = file;

        let form = try!(Blorb::load_chunk_data(&mut file));
        assert_eq!(&form.id, "FORM");

        // Check that the form type is IFRS
        let id = try!(Blorb::load_4bw(&mut file));
        assert_eq!(&id, "IFRS");

        let index = if let Chunk::ResourceIndex{index} =
                try!(Blorb::load_chunk(&mut file)) {
            index
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "could not locate index"
            ));
        };

        Ok(Blorb{
            len: form.len,
            index: index,
            file: file,
        })
    }

    /// loads a resource using the given index entry.
    pub fn load_resource(&mut self, num: usize) -> Result<Chunk, Error> {
        let entry = match self.index.entries.get(&num) {
            Some(entry) => entry,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "resource not found"
                ))
            },
        };
        try!(self.file.seek(SeekFrom::Start(entry.start as u64)));

        Blorb::load_chunk(&mut self.file)
    }

    fn load_chunk(file: &mut R) -> Result<Chunk, Error> {
        let meta = try!(Blorb::load_chunk_data(file));

        match meta.id.as_ref() {
            "RIdx" => {
                let num = try!(file.read_u32::<BigEndian>());
                let mut entries = HashMap::with_capacity(num as usize);
                for _ in 0..num {
                    let entry = try!(Blorb::load_index_entry(file));
                    entries.insert(entry.num as usize, entry);
                }

                Ok(Chunk::ResourceIndex{index: ResourceIndex{entries:entries}})
            },
            "GLUL" => {
                let mut data = Vec::with_capacity(meta.len as usize);
                try!(file.take(meta.len as u64).read_to_end(&mut data));
                Ok(Chunk::Glulx{code: data})
            },
            _ => {
                let mut data = Vec::with_capacity(meta.len as usize);
                try!(file.take(meta.len as u64).read_to_end(&mut data));
                Ok(Chunk::Unknown{meta: meta, data: data})
            },
        }
    }

    fn load_index_entry(file: &mut R) -> Result<IndexEntry, Error> {
        let usage = try!(Blorb::load_4bw(file));
        let num = try!(file.read_u32::<BigEndian>());
        let start = try!(file.read_u32::<BigEndian>());

        let usage = match usage.as_ref() {
            "Pict" => Usage::Pict,
            "Snd " => Usage::Snd,
            "Data" => Usage::Data,
            "Exec" => Usage::Exec,
            _ => return Err(Error::new(
                ErrorKind::InvalidInput,
                "could not identify index entry usage"
            )),
        };

        Ok(IndexEntry{usage: usage, num: num, start: start})
    }

    fn load_chunk_data(file: &mut R) -> Result<ChunkData, Error> {
        let id = try!(Blorb::load_4bw(file));
        let len = try!(file.read_u32::<BigEndian>());
        Ok(ChunkData{id: id, len: len})
    }

    /// Load a 4 byte ASCII word from the file
    fn load_4bw(file: &mut R) -> Result<String, Error> {
        let mut id = String::with_capacity(0x4);
        try!(file.take(4).read_to_string(&mut id));
        Ok(id)
    }
}
