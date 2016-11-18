use std::collections::HashMap;
use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Seek,
    SeekFrom
};

use byteorder::{BigEndian, ReadBytesExt};

use blorb::{
    Chunk,
    ChunkData,
    FormData,
    IndexEntry,
    ResourceIndex,
    Usage,
};


/// Provides access to blorb file contents without loading the full file
/// into memory.
///
/// When constructed, using the `BlorbCursor::from_file` method, the
/// `BlorbCursor` takes control of a struct implementing the
/// `std::io::Read` and `std::io::Seek` traits. The `BlorbCursor` then
/// validates this input blorb file, and loads the resource index and other
/// metadata objects.
///
/// When `BlorbCurosr::load_resource` is called, the given index is used
/// to lookup the starting location of the resource chunk in the file,
/// and seek to that location. Then, it loads the given resource from
/// the file and returns it to the caller.
pub struct BlorbCursor<R: Read + Seek + ?Sized> {
    /// The length of the blorb, minus the 8 byte chunk header.
    pub len: u32,
    index: ResourceIndex,
    file: R,
}


impl<R: Read + Seek> BlorbCursor<R> {

    /// Returns a `BlorbCursor` using the given blorb file. The blorb file
    /// is parsed and validated as part of this call. A `std::io::Error`
    /// is returned if an error occurs with accessing the file or if the
    /// file is invalid.
    pub fn from_file(src: R) -> Result<BlorbCursor<R>> {
        let mut src = src;

        // validate the file is a blorb form
        let form = (&mut src).read_form_data()?;
        if &form.id != b"IFRS" {
            return Err(Error::new(ErrorKind::InvalidInput,
                "file is not blorb"));
        }

        // validate the first chunk in the file is the index, and load
        // the index.
        if let Chunk::ResourceIndex{index} = src.read_chunk()? {
            Ok(BlorbCursor{len: form.len, index: index, file: src})
        } else {
            Err(Error::new(ErrorKind::InvalidInput,
                "blorb missing resource index"))
        }
    }

    /// Using the given index, looks up a blorb resource and load the
    /// resource chunk into memory. This chunk is then returned to the
    /// caller. An `std::io::Error` is returned if there is an exception
    /// while loading the resource into memory, if the loaded data is
    /// invalid, or if a resource is requested which is not identified
    /// in the `ResourceIndex`.
    pub fn load_resource(&mut self, index: u32) -> Result<Chunk> {
        let entry = match self.index.entries.get(&(index as usize)) {
            Some(entry) => entry,
            None => return Err(Error::new(ErrorKind::NotFound,
                "no entry associated with the given index")),
        };

        self.file.seek(SeekFrom::Start(entry.start as u64))?;
        (&mut self.file).read_chunk()
    }
}


/// An extension of the `std::io::Read` trait which reads blorb objects
/// from blorb files.
///
/// **TODO**: Eventually, this will be API -- so any internal methods
/// which are not offering up blorb structs will need to be moved.
trait ReadBlorbExt : Read {

    // Helper Methods
    ////////////////////////////////////////////////////////////////////
    // XXX: Find a better location for these methods

    /// Reads a 4 byte ASCII string into a `[u8; 0x4]`. Returns a
    /// `std::io::Error` if a problem arises reading the ascii bytes
    /// from the blorb.
    fn read_id(&mut self) -> Result<[u8; 0x4]> {
        let mut id = [0x0;0x4];
        self.read_exact(&mut id)?;
        Ok(id)
    }

    /// Light wrapper around the `std::io::Read::read_to_end` method
    /// which will return a `Vec` with `len` bytes from the file. If
    /// the number of bytes read does not match the expected length, or
    /// if other issues occur reading from the blorb, a `std::io::Error`
    /// is returned.
    fn read_exact_vec(&mut self, len: u32) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(len as usize);
        if len as usize != self.take(len as u64).read_to_end(&mut data)? {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "file ended before chunk fully read"));
        }
        Ok(data)
    }

    /// Light wrapper around the `std::io::Read::read_to_string` method
    /// which will return a `String` with `len` bytes from the file. If
    /// the number of bytes read does not match the expected length, or
    /// if other issues occur reading from the blorb, a `std::io::Error`
    /// is returned.
    fn read_exact_string(&mut self, len: u32) -> Result<String> {
        let mut data = String::with_capacity(len as usize);
        if len as usize != self.take(len as u64).read_to_string(&mut data)? {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "file ended before chunk fully read"));
        }
        Ok(data)
    }

    // Blorb metadata methods
    ////////////////////////////////////////////////////////////////////

    /// Reads the chunk metadata from the blorb. This moves the current
    /// position in the blorb forward by 8 bytes (so the next read will
    /// be at the chunk data). Returns an error if a failure occurred
    /// reading from the file.
    fn read_chunk_data(&mut self) -> Result<ChunkData> {
        Ok(ChunkData{id: self.read_id()?, len: self.read_u32::<BigEndian>()?})
    }

    /// Reads the form metadata from the blorb. This moves the current
    /// position in the blorb forward by 12 bytes (so the next read will
    /// be at the form data). Returns an error if a failure occurred
    /// reading from the file.
    fn read_form_data(&mut self) -> Result<FormData> {
        let meta = self.read_chunk_data()?;
        if &meta.id != b"FORM" {
            return Err(Error::new(ErrorKind::InvalidInput, "not FORM chunk"));
        }
        Ok(FormData{len: meta.len, id: self.read_id()?})
    }

    // Blorb Chunk methods
    ////////////////////////////////////////////////////////////////////

    /// Reads a `ChunkData` from the blorb. Then, uses that metadata to
    /// read the chunk data into a `Chunk`. Returns the chunk or the
    /// `std::io::Error` which occured when reading the chunk.`
    fn read_chunk(&mut self) -> Result<Chunk> {
        let meta = self.read_chunk_data()?;
        self.read_from_chunk_data(meta)
    }

    /// Takes a `ChunkData` and returns a `Chunk` based on the the
    /// metadata. Returns a `io::std::Error` if an issue occurs reading
    /// the data from the blorb.
    fn read_from_chunk_data(&mut self, meta: ChunkData) -> Result<Chunk> {
        match &meta.id {
            b"Fspc" => self.read_frontispiece(),
            b"GLUL" => self.read_glulx(meta.len),
            b"IFmd" => self.read_metadata(meta.len),
            b"JPEG" => self.read_jpeg(meta.len),
            b"PNG " => self.read_png(meta.len),
            b"RIdx" => self.read_resource_index(meta.len),
            _ => self.read_unknown(meta),
        }
    }

    //  Blorb Chunk variant methods
    ////////////////////////////////////////////////////////////////////
    // XXX: These functions should maybe be moved somewhere else before
    // this trait becomes public

    /// Read an index entry of a `ResourceIndex` from the blorb. return
    /// a `std::io::Error` if the blorb data is not valid.
    fn read_index_entry(&mut self) -> Result<IndexEntry> {
        let usage = match &self.read_id()? {
            b"Pict" => Usage::Pict,
            b"Snd " => Usage::Snd,
            b"Data" => Usage::Data,
            b"Exec" => Usage::Exec,
            _ => return Err(Error::new(ErrorKind::InvalidInput,
                "could not identify index entry usage")),
        };
        let num = self.read_u32::<BigEndian>()?;
        let start = self.read_u32::<BigEndian>()?;

        Ok(IndexEntry{usage: usage, num: num, start: start})
    }

    /// Read a `Chunk::ResourceIndex` data from the blorb file. Returns
    /// a `std::io::Error` if the blorb data is not valid.
    fn read_resource_index(&mut self, len: u32) -> Result<Chunk> {
        let num = self.read_u32::<BigEndian>()?;

        // validate resource index length
        if len != num*12 + 4 {
            return Err(Error::new(ErrorKind::InvalidInput,
                "length of resource index does not match item length"));
        }

        // retrieve entries and store in hashmap based on index
        let mut entries = HashMap::with_capacity(num as usize);
        for _ in 0..num {
            let entry = self.read_index_entry()?;
            entries.insert(entry.num as usize, entry);
        }
        let entries = entries;

        Ok(Chunk::ResourceIndex{index: ResourceIndex{entries: entries}})
    }

    /// Read a `Chunk::Glulx` data from the blorb file. Returns
    /// a `std::io::Error` if the blorb data is not valid.
    fn read_glulx(&mut self, len: u32) -> Result<Chunk> {
        let code = self.read_exact_vec(len)?;
        if len & 1 == 1 {self.read_exact(&mut [0x0])?};
        Ok(Chunk::Glulx{code: code})
    }

    /// Read a `Chunk::Frontispiece` data from the blorb file. Returns
    /// a `std::io::Error` if the blorb data is not valid.
    fn read_frontispiece(&mut self) -> Result<Chunk> {
        Ok(Chunk::Frontispiece{num: self.read_u32::<BigEndian>()?})
    }

    /// Read a `Chunk::Metadata` data from the blorb file. Returns
    /// a `std::io::Error` if the blorb data is not valid.
    fn read_metadata(&mut self, len: u32) -> Result<Chunk> {
        let info = self.read_exact_string(len)?;
        if len & 1 == 1 {self.read_exact(&mut [0x0])?};
        Ok(Chunk::Metadata{info: info})
    }

    /// Read a `Chunk::Png` data from the blorb file. Returns
    /// a `std::io::Error` if the blorb data is not valid.
    fn read_png(&mut self, len: u32) -> Result<Chunk> {
        let data = self.read_exact_vec(len)?;
        if len & 1 == 1 {self.read_exact(&mut [0x0])?};
        Ok(Chunk::Png{data: data})
    }

    /// Read a `Chunk::Jpeg` data from the blorb file. Returns
    /// a `std::io::Error` if the blorb data is not valid.
    fn read_jpeg(&mut self, len: u32) -> Result<Chunk> {
        let data = self.read_exact_vec(len)?;
        if len & 1 == 1 {self.read_exact(&mut [0x0])?};
        Ok(Chunk::Jpeg{data: data})
    }

    /// Read a `Chunk::Unknown` from the blorb file. Returns
    /// a `std::io::Error` if the blorb data is not valid.
    fn read_unknown(&mut self, meta: ChunkData) -> Result<Chunk> {
        let data = self.read_exact_vec(meta.len)?;
        if meta.len & 1 == 1 {self.read_exact(&mut [0x0])?};
        Ok(Chunk::Unknown{meta: meta, data: data})
    }
}


impl<R: Read + ?Sized> ReadBlorbExt for R {}
