use std::error::Error;
use std::path::{Path, PathBuf};
use naga::WithSpan;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

#[macro_export]
macro_rules! preprocess_shader {
    ($source: expr) => {
        {
            let mut source: String = $source;

            for line in source.lines().map(|x| x.to_string()).collect::<Vec<String>>().iter() {
                if line.starts_with("%include") {
                    let file_split = line.split(" ").map(|x| x.to_string()).collect::<Vec<String>>();
                    let file = file_split.get(1).unwrap();
                    let file_source = std::fs::read_to_string(file)
                        .expect(format!("Source included file {} doesn't exist!", file).as_str());

                    source = source.replace(line, &file_source);
                }
            }

            source
        }
    }
}

fn emit_glsl_parser_error(errors: Vec<naga::front::glsl::Error>, filename: &str, source: &str) {
    let files = SimpleFile::new(filename, source);
    let config = codespan_reporting::term::Config::default();
    let writer = StandardStream::stderr(ColorChoice::Auto);

    for err in errors {
        let mut diagnostic = Diagnostic::error().with_message(err.kind.to_string());

        if let Some(range) = err.meta.to_range() {
            diagnostic = diagnostic.with_labels(vec![Label::primary((), range)]);
        }

        term::emit(&mut writer.lock(), &config, &files, &diagnostic).expect("cannot write error");
    }
}

fn emit_annotated_error<E: Error>(ann_err: &WithSpan<E>, filename: &str, source: &str) {
    let files = SimpleFile::new(filename, source);
    let config = codespan_reporting::term::Config::default();
    let writer = StandardStream::stderr(ColorChoice::Auto);

    let diagnostic = Diagnostic::error().with_labels(
        ann_err
            .spans()
            .map(|(span, desc)| {
                Label::primary((), span.to_range().unwrap()).with_message(desc.to_owned())
            })
            .collect(),
    );

    term::emit(&mut writer.lock(), &config, &files, &diagnostic).expect("cannot write error");
}

fn print_err(error: &dyn Error) {
    eprint!("{}", error);

    let mut e = error.source();
    if e.is_some() {
        eprintln!(": ");
    } else {
        eprintln!();
    }

    while let Some(source) = e {
        eprintln!("\t{}", source);
        e = source.source();
    }
}

fn get_output_path() -> PathBuf {
    let manifest_dir_string = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = std::env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);

    PathBuf::from(path)
}

fn get_shader_output_path(build_path: PathBuf) -> PathBuf {
    let mut shader_output = build_path.parent().unwrap().to_path_buf();

    shader_output.push("build");

    shader_output
}

fn compile_source(stage: naga::ShaderStage, source: &str) {
    let validation_caps = naga::valid::Capabilities::CLIP_DISTANCE | naga::valid::Capabilities::CULL_DISTANCE;
    let shader_output_path = get_shader_output_path(get_output_path());
    let mut parser = naga::front::glsl::Parser::default();

    let mut shader_output_path = shader_output_path.clone();
    shader_output_path.set_file_name(source.split("/").last().unwrap());
    shader_output_path.set_extension("spv");

    println!("Operating on {} ({:?})", source, stage);

    /* let mut preprocessed_shader_source_path = shader_output_path.clone();
    preprocessed_shader_source_path.set_file_name(compute_source.split("/").last().unwrap().replace(".comp", ".tmp.comp"));

    std::fs::write(&preprocessed_shader_source_path, preprocess_shader!(std::fs::read_to_string(compute_source).unwrap())).unwrap();

    let status = std::process::Command::new("glslc")
        // .arg("-fshader-stage").arg("compute")
        .arg("-x").arg("glsl")
        .arg("-c").arg(&preprocessed_shader_source_path)
        .arg("-o").arg(shader_output_path)
        .status()
        .expect("Failed to execute shader compilation (compute GLSL -> SPIR-V) process!");

    if ! status.success() {
        std::process::exit(1);
    }

    std::fs::remove_file(preprocessed_shader_source_path).unwrap(); */

    let input = preprocess_shader!(std::fs::read_to_string(source).unwrap());

    let module = parser.parse(&naga::front::glsl::Options {
        stage,
        defines: Default::default(),
    }, &input).unwrap_or_else(|errors| {
        emit_glsl_parser_error(errors, "terrain.comp", &input);
        std::process::exit(1);
    });

    let info = match naga::valid::Validator::new(naga::valid::ValidationFlags::empty(), validation_caps).validate(&module) {
        Ok(info) => info,
        Err(error) => {
            emit_annotated_error(&error, source, &input);
            print_err(&error);
            std::process::exit(1);
        }
    };

    /* let wgsl = naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty()).unwrap();

    std::fs::write(shader_output_path.to_str().unwrap(), wgsl).unwrap(); */

    let pipeline_options_owned;
    let pipeline_options = {
        let ep_index = module
            .entry_points
            .iter()
            .position(|ep| ep.name == "main")
            .expect("Unable to find the entry point aptly named `main'.");

        pipeline_options_owned = naga::back::spv::PipelineOptions {
            entry_point: "main".to_owned(),
            shader_stage: module.entry_points[ep_index].stage,
        };

        &pipeline_options_owned
    };

    let mut options = naga::back::spv::Options::default();

    options.bounds_check_policies = naga::proc::BoundsCheckPolicies::default();
    options.flags.set(
        naga::back::spv::WriterFlags::ADJUST_COORDINATE_SPACE,
        !false,
    );

    let output = naga::back::spv::write_vec(&module, &info, &options, Some(pipeline_options)).unwrap();
    let bytes = output.iter()
        .fold(Vec::with_capacity(output.len() * 4), |mut v, w| {
            v.extend_from_slice(&w.to_le_bytes());
            v
        });

    std::fs::write(shader_output_path, bytes.as_slice()).unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./Cargo.lock");

    let sane_shader_extensions = vec!["vert", "frag"];
    for element in std::path::Path::new(r"./res/shaders/").read_dir().unwrap() {
        let path = &element.unwrap().path();
        if let Some(extension) = path.extension() {
            let extension_str = extension.to_str().unwrap();
            if ! path.file_name().unwrap().to_str().unwrap().starts_with("h_") &&
                sane_shader_extensions.contains(&extension_str) {
                println!("cargo:rerun-if-changed={}", path.display());

                let shader_stage = match extension_str {
                    "vert" => naga::ShaderStage::Vertex,
                    "frag" => naga::ShaderStage::Fragment,
                    _ => unreachable!(),
                };

                compile_source(shader_stage, path.to_str().unwrap());
            }
        }
    }
}

