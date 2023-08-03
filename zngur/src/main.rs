use std::{fs::File, io::Write};

// use ra_ap_hir::db::DefDatabase;
// use ra_ap_hir_def::ModuleId;
// use ra_ap_hir_ty::{db::HirDatabase, AdtId, Interner, Substitution, TyKind};
// use ra_ap_ide::AnalysisHost;
// use ra_ap_ide_db::base_db::SourceDatabase;
// use ra_ap_load_cargo::{load_workspace, LoadCargoConfig, ProcMacroServerChoice};
// use ra_ap_paths::AbsPath;
// use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use zngur_generator::{RustType, ZngurFile, ZngurMethod, ZngurMethodReceiver, ZngurType};

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
            let Some(n) = &db.crate_graph()[*x].display_name else { return false };
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

fn main() {
    let file = ZngurFile {
        types: vec![
            ZngurType {
                ty: RustType::from("::crate::St"),
                size: 32,
                align: 8,
                is_copy: false,
                methods: vec![],
            },
            ZngurType {
                ty: RustType::from("()"),
                size: 0,
                align: 1,
                is_copy: true,
                methods: vec![],
            },
            ZngurType {
                ty: RustType::from("Box<dyn Fn(i32) -> i32>"),
                size: 16,
                align: 8,
                is_copy: false,
                methods: vec![],
            },
            ZngurType {
                ty: RustType::from(
                    "::std::iter::Map<::std::vec::IntoIter<i32>, Box<dyn Fn(i32) -> i32>>",
                ),
                size: 48,
                align: 8,
                is_copy: false,
                methods: vec![ZngurMethod {
                    name: "sum".to_owned(),
                    generics: vec![RustType::from("i32")],
                    receiver: ZngurMethodReceiver::Move,
                    inputs: vec![],
                    output: RustType::from("i32"),
                }],
            },
            ZngurType {
                ty: RustType::from("::std::vec::IntoIter<i32>"),
                size: 32,
                align: 8,
                is_copy: false,
                methods: vec![
                    ZngurMethod {
                        name: "sum".to_owned(),
                        generics: vec![RustType::from("i32")],
                        receiver: ZngurMethodReceiver::Move,
                        inputs: vec![],
                        output: RustType::from("i32"),
                    },
                    ZngurMethod {
                        name: "map".to_owned(),
                        generics: vec![
                            RustType::from("i32"),
                            RustType::from("Box<dyn Fn(i32) -> i32>"),
                        ],
                        receiver: ZngurMethodReceiver::Move,
                        inputs: vec![RustType::from("Box<dyn Fn(i32) -> i32>")],
                        output: RustType::from(
                            "::std::iter::Map<::std::vec::IntoIter<i32>, Box<dyn Fn(i32) -> i32>>",
                        ),
                    },
                ],
            },
            ZngurType {
                ty: RustType::from("::std::vec::Vec<i32>"),
                size: 24,
                align: 8,
                is_copy: false,
                methods: vec![
                    ZngurMethod {
                        name: "new".to_owned(),
                        generics: vec![],
                        receiver: ZngurMethodReceiver::Static,
                        inputs: vec![],
                        output: RustType::from("::std::vec::Vec<i32>"),
                    },
                    ZngurMethod {
                        name: "push".to_owned(),
                        generics: vec![],
                        receiver: ZngurMethodReceiver::RefMut,
                        inputs: vec![RustType::from("i32")],
                        output: RustType::from("()"),
                    },
                    ZngurMethod {
                        name: "clone".to_owned(),
                        generics: vec![],
                        receiver: ZngurMethodReceiver::Ref,
                        inputs: vec![],
                        output: RustType::from("::std::vec::Vec<i32>"),
                    },
                    ZngurMethod {
                        name: "into_iter".to_owned(),
                        generics: vec![],
                        receiver: ZngurMethodReceiver::Move,
                        inputs: vec![],
                        output: RustType::from("::std::vec::IntoIter<i32>"),
                    },
                ],
            },
        ],
    };

    let (rust, cpp) = file.render();
    File::create("../simple-example/src/generated.rs")
        .unwrap()
        .write_all(rust.as_bytes())
        .unwrap();
    File::create("../simple-example/generated.h")
        .unwrap()
        .write_all(cpp.as_bytes())
        .unwrap();
}
