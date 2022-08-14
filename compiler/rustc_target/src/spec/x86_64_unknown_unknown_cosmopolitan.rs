use crate::spec::{
    crt_objects, cvs, FramePointer, LinkerFlavor, PanicStrategy, RelocModel, StackProbeType,
    Target, TargetOptions,
};

const LINKER_SCRIPT: &str = include_str!("./x86_64_unknown_unknown_cosmopolitan_linker_script.ld");

pub fn target() -> Target {
    Target {
        llvm_target: "x86_64-unknown-unknown-cosmopolitan".into(),
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            .into(),
        arch: "x86_64".into(),
        options: TargetOptions {
            os: "unknown".into(),
            env: "cosmopolitan".into(),
            is_builtin: true,
            linker_is_gnu: true,
            linker_flavor: LinkerFlavor::Gcc,
            link_script: Some(LINKER_SCRIPT.into()),
            cpu: "x86-64".into(),
            relocation_model: RelocModel::Static,
            disable_redzone: true,
            frame_pointer: FramePointer::Always,
            exe_suffix: "com.dbg".into(),
            max_atomic_width: Some(64),
            panic_strategy: PanicStrategy::Abort,
            stack_probes: StackProbeType::None,
            crt_static_default: true,
            post_link_objects: crt_objects::post_cosmopolitan(),
            post_link_objects_fallback: crt_objects::post_cosmopolitan_fallback(),
            requires_uwtable: false,
            has_rpath: false,
            dynamic_linking: false,
            executables: true,
            position_independent_executables: false,
            static_position_independent_executables: false,
            has_thread_local: true,
            eh_frame_header: false,
            no_default_libraries: true,
            families: cvs!["unix"],
            ..Default::default()
        },
    }
}
