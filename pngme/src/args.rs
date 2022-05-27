use std::path::PathBuf;

pub enum PngMeArgs {
   Encode(EncodeArgs), 
   Decode(DecodeArgs),
   Remove(RemoveArgs),
   Print(PrintArgs),
}


pub struct EncodeArgs {
    pub path:PathBuf,
    pub chunk_type:String,
    pub data:String,
}

pub struct DecodeArgs {
    pub path:PathBuf,
    pub chunk_type :String,
}

pub struct RemoveArgs{
    pub path:PathBuf,
    pub chunk_type:String,
}

pub struct PrintArgs{
    pub path:PathBuf,
}