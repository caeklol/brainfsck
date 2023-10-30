use clap::{Parser, Subcommand};
use std::{fs, io::Write, path::Path};

#[cfg(test)]
mod test;

mod parser;

mod compiler;
mod runner;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[clap(default_value_t = 30000)]
    tape_size: usize,
}

#[derive(Subcommand)]
enum Commands {
    Interpret { path: String },
    Compile { path: String, out_path: String },
}

fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Interpret { path } => {
            let file_path = fs::canonicalize(Path::new(&path))?;
            let bf_code = fs::read_to_string(file_path)?;

            let mut env = runner::Environment::new(cli.tape_size);
            env.evaluate(&bf_code)?;
        }

        Commands::Compile { path, out_path } => {
            let cur_dir = std::env::current_dir()?;
            let normalized_out_path = path_clean::clean(cur_dir.join(out_path));
            let normalized_out_path_str = normalized_out_path.to_string_lossy();
            let file_path = fs::canonicalize(Path::new(&path))?;

            let bf_code = fs::read_to_string(&file_path)?;
            let binary_data =
                compiler::gen_object(&bf_code, cli.tape_size).expect("Failed to compile!");

            let mut temp_file = tempfile::NamedTempFile::new()?;
            temp_file.write_all(binary_data.as_slice())?;

            let temp_path = temp_file.into_temp_path();
            let temp_path_str: &str = &temp_path.to_string_lossy();

            std::process::Command::new("cc")
                .arg("-O4")
                .arg(temp_path_str)
                .args(["-o", &normalized_out_path_str])
                .spawn()?
                .wait()?;

            temp_path.close()?;
        }
    }

    Ok(())
}
