use clap::{Parser, Args, Subcommand};

#[derive(Parser, Debug)]
#[command()]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Encode(EncodeArgs),
    Decode(DecodeArgs),
    Remove(RemoveArgs),
    Print(PrintArgs)
}
#[derive(Args, Debug)]
pub struct EncodeArgs {
    filepath: String,
    chunk_type: String,
    message: String,
    output: Option<String>
}
#[derive(Args, Debug)]
pub struct DecodeArgs {
    filepath: String,
    chunk_type: String,
}
#[derive(Args, Debug)]
pub struct RemoveArgs {
    filepath: String,
    chunk_type: String,
}
#[derive(Args, Debug)]
pub struct PrintArgs {
    filepath: String,
}