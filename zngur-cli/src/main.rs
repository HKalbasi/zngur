use std::path::PathBuf;

use clap::Parser;
use zngur::Zngur;

#[derive(Parser)]
#[command(version)]
enum Command {
    #[command(alias = "g")]
    Generate {
        /// Path to the zng file
        path: PathBuf,

        /// Path of the generated C++ file, if it is needed
        ///
        /// Default is {ZNG_FILE_PARENT}/generated.cpp
        #[arg(long)]
        cpp_file: Option<PathBuf>,

        /// Path of the generated header file
        ///
        /// Default is {ZNG_FILE_PARENT}/generated.h
        #[arg(long)]
        h_file: Option<PathBuf>,

        /// Path of the generated Rust file
        ///
        /// Default is {ZNG_FILE_PARENT}/src/generated.rs
        #[arg(long)]
        rs_file: Option<PathBuf>,

        /// A unique string which is included in zngur symbols to prevent duplicate
        /// symbols in linker
        ///
        /// Default is the value of cpp_namespace, so you don't need to set this manually
        /// if you change cpp_namespace as well
        #[arg(long)]
        mangling_base: Option<String>,

        /// The C++ namespace which zngur puts its things in it. You can change it
        /// to prevent violation of ODR when you have multiple independent zngur
        /// libraries
        ///
        /// Default is "rust"
        #[arg(long)]
        cpp_namespace: Option<String>,
    },

    /// Extract and display layout information for types with #layout(auto)
    #[command(alias = "dl")]
    DumpLayouts {
        /// Path to the zng file
        path: PathBuf,

        /// Target triple for cross-compilation (e.g., x86_64-unknown-linux-gnu)
        #[arg(long)]
        target: Option<String>,

        /// Path to the crate containing the types (default: current directory)
        #[arg(long)]
        crate_path: Option<PathBuf>,
    },

    /// Generate a standalone bridge project
    #[command(alias = "gs")]
    GenerateStandalone {
        /// Path to the zng file
        path: PathBuf,

        /// Output directory name for the bridge project
        ///
        /// Default is "zngur-bridge"
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// A unique string which is included in zngur symbols to prevent duplicate
        /// symbols in linker
        #[arg(long)]
        mangling_base: Option<String>,

        /// The C++ namespace which zngur puts its things in it
        ///
        /// Default is "rust"
        #[arg(long)]
        cpp_namespace: Option<String>,
    },
}

fn main() {
    let cmd = Command::parse();
    match cmd {
        Command::Generate {
            path,
            cpp_file,
            h_file,
            rs_file,
            mangling_base,
            cpp_namespace,
        } => {
            let pp = path.parent().unwrap();
            let cpp_file = cpp_file.unwrap_or_else(|| pp.join("generated.cpp"));
            let h_file = h_file.unwrap_or_else(|| pp.join("generated.h"));
            let rs_file = rs_file.unwrap_or_else(|| pp.join("src/generated.rs"));
            let mut zng = Zngur::from_zng_file(&path)
                .with_cpp_file(cpp_file)
                .with_h_file(h_file)
                .with_rs_file(rs_file);
            if let Some(mangling_base) = mangling_base {
                zng = zng.with_mangling_base(&mangling_base);
            }
            if let Some(cpp_namespace) = cpp_namespace {
                zng = zng.with_cpp_namespace(&cpp_namespace);
            }
            zng.generate();
        }
        Command::DumpLayouts {
            path,
            target,
            crate_path,
        } => {
            use zngur_auto_layout::LayoutExtractor;
            use zngur_parser::ParsedZngFile;

            // Parse the zng file to find types with Auto layout
            let spec = ParsedZngFile::parse(path.clone());

            let auto_types: Vec<_> = spec
                .types
                .iter()
                .filter(|ty_def| matches!(ty_def.layout, zngur_def::LayoutPolicy::Auto))
                .map(|ty_def| ty_def.ty.clone())
                .collect();

            if auto_types.is_empty() {
                println!("No types with #layout(auto) found in {}", path.display());
                return;
            }

            let crate_path = crate_path.unwrap_or_else(|| std::env::current_dir().unwrap());

            let mut extractor = LayoutExtractor::new(&crate_path);
            if let Some(target) = target {
                extractor = extractor.with_target(target);
            }

            match extractor.dump_layouts_zng(&auto_types) {
                Ok(output) => println!("{}", output),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::GenerateStandalone {
            path,
            output,
            mangling_base,
            cpp_namespace,
        } => {
            let output_dir = output.unwrap_or_else(|| PathBuf::from("zngur-bridge"));

            let mut zng = Zngur::from_zng_file(&path).standalone_mode(output_dir);

            if let Some(mangling_base) = mangling_base {
                zng = zng.with_mangling_base(&mangling_base);
            }
            if let Some(cpp_namespace) = cpp_namespace {
                zng = zng.with_cpp_namespace(&cpp_namespace);
            }

            zng.generate();
        }
    }
}
