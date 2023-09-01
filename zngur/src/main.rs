use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;
// use ra_ap_hir::db::DefDatabase;
// use ra_ap_hir_def::ModuleId;
// use ra_ap_hir_ty::{db::HirDatabase, AdtId, Interner, Substitution, TyKind};
// use ra_ap_ide::AnalysisHost;
// use ra_ap_ide_db::base_db::SourceDatabase;
// use ra_ap_load_cargo::{load_workspace, LoadCargoConfig, ProcMacroServerChoice};
// use ra_ap_paths::AbsPath;
// use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use zngur_generator::{ParsedZngFile, ZngurGenerator};

#[cfg(False)]
fn parse_repo() {
    let path = "../simple-example";
    let path = AbsPath::absolutize(
        AbsPath::assert(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        path,
    );
    let manifest = ProjectManifest::discover_single(&path).unwrap();
    let mut cargo_config = CargoConfig::default();
    cargo_config.sysroot = Some(RustLibSource::Discover);
    let workspace = ProjectWorkspace::load(manifest, &cargo_config, &|_| ()).unwrap();

    let load_cargo_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::Sysroot,
        prefill_caches: false,
    };

    let (ah, _, _) = load_workspace(workspace, &HashMap::default(), &load_cargo_config).unwrap();

    let db = ah.raw_database();

    let krate = db
        .crate_graph()
        .iter()
        .find(|x| {
            let Some(n) = &db.crate_graph()[*x].display_name else {
                return false;
            };
            n.canonical_name() == "simple-example"
        })
        .unwrap();
    let module_id: ModuleId = db.crate_def_map(krate).crate_root().into();
    let def_map = module_id.def_map(db);
    let scope = &def_map[module_id.local_id].scope;
    let st_struct = scope
        .declarations()
        .find_map(|x| {
            dbg!(x);
            if let ra_ap_hir_def::ModuleDefId::AdtId(x) = x {
                if let ra_ap_hir_def::AdtId::StructId(x) = x {
                    return Some(x);
                }
            }
            None
        })
        .unwrap();
    let st_ty =
        TyKind::Adt(AdtId(st_struct.into()), Substitution::empty(Interner)).intern(Interner);
    let layout = db
        .layout_of_ty(st_ty, db.trait_environment(st_struct.into()))
        .unwrap();
    dbg!(layout);
}

#[derive(Parser)]
enum Command {
    #[command(alias = "g")]
    Generate { path: PathBuf },
}

fn main() {
    let cmd = Command::parse();
    match cmd {
        Command::Generate { path } => {
            let file = std::fs::read_to_string(&path).unwrap();
            let file =
                ParsedZngFile::parse("main.zng", &file, |f| ZngurGenerator::build_from_zng(f));

            let (rust, h, cpp) = file.render();
            let path = path.parent().unwrap();
            File::create(path.join("src/generated.rs"))
                .unwrap()
                .write_all(rust.as_bytes())
                .unwrap();
            File::create(path.join("generated.h"))
                .unwrap()
                .write_all(h.as_bytes())
                .unwrap();
            if let Some(cpp) = cpp {
                File::create(path.join("generated.cpp"))
                    .unwrap()
                    .write_all(cpp.as_bytes())
                    .unwrap();
            }
        }
    }
}
