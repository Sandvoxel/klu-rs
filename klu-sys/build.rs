use std::path::Path;

fn main() {
    if std::env::var_os("CARGO_FEATURE_DYNAMIC").is_some() {
        println!("cargo:rustc-link-lib=klu");
    } else {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor");
        build_suitesparse_config(&src_dir);
        for &long in &[false, true] {
            build_amd(&src_dir, long);
            build_colamd(&src_dir, long);
            build_btf(&src_dir, long);
            build_klu_common(&src_dir, long);
            for &complex in &[false, true] {
                build_klu(&src_dir, long, complex);
            }
        }
    }
}

fn build_suitesparse_config(src_dir: &Path) {
    let mut builder = cc::Build::new();

    if std::env::var("CARGO_CFG_TARGET_OS")
        .map(|os| os == "windows")
        .unwrap_or(false)
    {
        builder.define("NTIMER", None);
    }

    builder
        .file(
            src_dir
                .join("SuiteSparse_config")
                .join("SuiteSparse_config.c"),
        )
        .include(src_dir.join("SuiteSparse_config").join("Config"))
        .compile("suitesparseconfig");
}

fn setup_suitesparse_builder(src_dir: &Path) -> cc::Build {
    let mut builder = cc::Build::new();

    builder
        .include(src_dir.join("SuiteSparse_config"))
        .include(src_dir.join("SuiteSparse_config").join("Config"));
    builder
}

fn build_amd(src_dir: &Path, long: bool) {
    let mut builder = setup_suitesparse_builder(src_dir);
    let dir = src_dir.join("AMD");
    let amd_src_dir = dir.join("Source");

    builder.include(dir.join("Include"));

    let amd_objects: &[&str] = if long {
        &[
            "amd_l1",
            "amd_l2",
            "amd_l_aat",
            "amd_l_control",
            "amd_l_defaults",
            "amd_l_dump",
            "amd_l_info",
            "amd_l_order",
            "amd_l_post_tree",
            "amd_l_postorder",
            "amd_l_preprocess",
            "amd_l_valid",
            "amd_version",
        ]
    } else {
        &[
            "amd_1",
            "amd_2",
            "amd_aat",
            "amd_control",
            "amd_defaults",
            "amd_dump",
            "amd_info",
            "amd_order",
            "amd_post_tree",
            "amd_postorder",
            "amd_preprocess",
            "amd_valid",
            "amd_version",
        ]
    };

    for obj in amd_objects {
        builder.file(amd_src_dir.join(format!("{obj}.c")));
    }

    builder.compile(if long { "amdl" } else { "amd" });
}

fn build_colamd(src_dir: &Path, long: bool) {
    let mut builder = setup_suitesparse_builder(src_dir);

    let dir = src_dir.join("COLAMD");
    let colamd_src_dir = dir.join("Source");
    let src = if long {
        colamd_src_dir.join("colamd_l.c")
    } else {
        colamd_src_dir.join("colamd.c")
    };
    builder.include(dir.join("Include")).file(src);
    builder.file(colamd_src_dir.join("colamd_version.c"));

    builder.compile(if long { "colamdl" } else { "colamd" });
}

fn build_btf(src_dir: &Path, long: bool) {
    let mut builder = setup_suitesparse_builder(src_dir);

    let dir = src_dir.join("BTF");
    let btf_src_dir = dir.join("Source");
    builder.include(dir.join("Include"));

    let btf_objects: &[&str] = if long {
        &[
            "btf_l_maxtrans",
            "btf_l_order",
            "btf_l_strongcomp",
            "btf_version",
        ]
    } else {
        &["btf_maxtrans", "btf_order", "btf_strongcomp", "btf_version"]
    };
    for obj in btf_objects {
        builder.file(btf_src_dir.join(format!("{obj}.c")));
    }

    builder.compile(if long { "btfl" } else { "btf" });
}

fn build_klu_common(src_dir: &Path, long: bool) {
    let klu_src = src_dir.join("KLU").join("Source");

    let mut builder = setup_suitesparse_builder(src_dir);
    let prefix = if long { "klu_l" } else { "klu" };
    let objects = [
        "_analyze",
        "_analyze_given",
        "_defaults",
        "_memory",
        "_free_symbolic",
    ];

    for obj in &objects {
        builder.file(klu_src.join(format!("{prefix}{obj}.c")));
    }

    builder
        .include(src_dir.join("KLU").join("Include"))
        .include(src_dir.join("AMD").join("Include"))
        .include(src_dir.join("COLAMD").join("Include"))
        .include(src_dir.join("BTF").join("Include"))
        .include(src_dir.join("SuiteSparse_config"))
        .compile(if long { "klul_common" } else { "klu_common" });
}

fn build_klu(src_dir: &Path, long: bool, complex: bool) {
    let klu_src = src_dir.join("KLU").join("Source");

    let mut builder = setup_suitesparse_builder(src_dir);

    let prefix = match (complex, long) {
        (false, false) => "klu",
        (false, true) => "klu_l",
        (true, false) => "klu_z",
        (true, true) => "klu_zl",
    };

    let lib_name = format!(
        "klu{}",
        match (complex, long) {
            (false, false) => "",
            (false, true) => "l",
            (true, false) => "z",
            (true, true) => "zl",
        }
    );

    let objects = [
        "",
        "_diagnostics",
        "_dump",
        "_extract",
        "_factor",
        "_free_numeric",
        "_kernel",
        "_refactor",
        "_scale",
        "_solve",
        "_sort",
        "_tsolve",
    ];

    for obj in &objects {
        builder.file(klu_src.join(format!("{prefix}{obj}.c")));
    }

    if !long && !complex {
        builder.file(klu_src.join("klu_version.c"));
    }

    builder
        .include(src_dir.join("KLU").join("Include"))
        .include(src_dir.join("AMD").join("Include"))
        .include(src_dir.join("COLAMD").join("Include"))
        .include(src_dir.join("BTF").join("Include"))
        .include(src_dir.join("SuiteSparse_config"))
        .include(src_dir.join("SuiteSparse_config").join("Config"))
        .compile(&lib_name);
}
