use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::{
    f32::{self},
    path::Path,
};

use ace::gfx;
use clap::{Parser, Subcommand};

use crate::ibl::Baker;

mod ibl;
mod units;

#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    action: Action,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Action {
    Bake {
        #[arg()]
        input_path: PathBuf,
        /// Sample delta for convolution. Lower value increases precision and runtime
        #[arg(short, long, default_value_t = 0.025)]
        convolute_sample_delta: f32,
        /// Sample count for brdf textures. Higher values increase precision and runtime
        #[arg(short, long, default_value_t = 1024)]
        brdf_sample_count: u32,
        #[arg(short, long, default_value = "skybox.ibl")]
        output_path: PathBuf,
        /// Number of (logical) cores to use.
        /// By default tries to utilize 100% of the CPU
        #[arg(long)]
        cores: Option<usize>,
    },
    Inspect {
        #[arg()]
        input_path: PathBuf,
    },
    Hello {
        #[arg()]
        name: String,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_thread_ids(true)
        .with_target(false)
        .compact()
        .init();
    let command = Command::parse();
    match command.action {
        Action::Bake {
            input_path: path,
            cores,
            convolute_sample_delta,
            brdf_sample_count,
            output_path,
        } => {
            let ibl = bake(&path, cores, convolute_sample_delta, brdf_sample_count);
            let data = ibl.serialize();
            let mut output =
                File::create(&output_path).expect("Failed to create empty skybox.ibl file");
            output
                .write_all(&data)
                .unwrap_or_else(|_| panic!("Failed to write {}", output_path.display()));
        }
        Action::Inspect { input_path } => inspect(&input_path),
        Action::Hello { name } => println!("Hello, {name}!"),
    }
}

fn bake(
    path: &Path,
    cores: Option<usize>,
    convolute_sample_delta: f32,
    brdf_sample_count: u32,
) -> gfx::Ibl {
    let baker = ibl::CpuBaker::new(
        cores.unwrap_or(1),
        convolute_sample_delta,
        brdf_sample_count,
    );
    let image = image::ImageReader::open(path).expect("failed to open image");
    let image = image.decode().expect("failed to decode image");
    baker.bake(Arc::new(image))
}

fn inspect(input_path: &Path) {
    let file = fs::read(input_path).expect("failed to read file");
    let skybox = gfx::Ibl::deserialize(&file);
    let json = serde_json::json!(skybox);

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
