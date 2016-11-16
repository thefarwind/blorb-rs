use std::collections::HashMap;

// Metadata Structs
////////////////////////////////////////////////////////////////////////

/// Metadata struct for `FORM` chunks. Contains the length of the chunk
/// and the form id of the chunk. Provides slightly more info into a
/// `FORM` chunk so that use and contained data can be determined.
///
/// **NOTE**: The `len` includes the 4 bytes in `id`. The remaining
/// length of the chunk after the `id` is `len - 4`.
#[derive(Debug)]
pub struct FormData {
    pub len: u32,
    pub id: [u8; 0x4],
}


/// Container for chunk metadata. Used for identifying a chunk without
/// loading the full chunk into memory.
#[derive(Debug)]
pub struct ChunkData {
    pub id: [u8; 0x4],
    pub len: u32,
}


impl From<FormData> for ChunkData {
    fn from(form_data: FormData) -> ChunkData {
        ChunkData{len: form_data.len, id: *b"FORM"}
    }
}


// Chunk Structs
////////////////////////////////////////////////////////////////////////


/// The usage information for an `IndexEntry`.
#[derive(Debug)]
pub enum Usage {
    /// Identifier: 'Pict'.
    /// Indicates the resource is an image.
    Pict,

    /// Identifier: 'Snd '.
    /// Indicates the resource is a sound.
    Snd,

    /// Indentifier: 'Data'.
    /// Indicates the resource is some data.
    Data,

    /// Identifier: 'Exec'.
    /// Indicates the resource is an executable.
    Exec,
}


/// Contains the usage information for an entry, the resource number of
/// the entry, and where in the blob the entry starts.
#[derive(Debug)]
pub struct IndexEntry {
    pub usage: Usage,
    pub num: u32,
    pub start: u32,
}


/// Container for list of resource index entries.
#[derive(Debug)]
pub struct ResourceIndex {
    pub entries: HashMap<usize, IndexEntry>,
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

    /// Identifier: 'IFmd'.
    /// Contains xml metadata content for the IF.
    Metadata{info: String},

    /// Identifier: 'Fspec'.
    /// Contains a reference to a frontispiece image.
    /// This chunk is optional.
    Frontispiece{num: u32},

    /// Identifier: 'GLUL'.
    /// Contains Glulx executable.
    /// This is a executable resource chunk.
    Glulx{code: Vec<u8>},

    /// Identifier: 'PNG '.
    /// Contains a PNG image.
    /// This is a picture resource chunk.
    Png{data: Vec<u8>},

    /// Identifier: 'JPEG'.
    /// Contains a JPEG image.
    /// This is a picture resource chunk.
    Jpeg{data: Vec<u8>},
}
