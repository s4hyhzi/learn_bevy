use naga::ShaderStage;
use naga::front::glsl::{Frontend, Options};
use naga::valid::{Capabilities, Validator};
use naga::valid::ValidationFlags;
use naga::back::wgsl;
pub fn glsl_to_wgsl(glsl: &str, stage: ShaderStage) -> String {
    let mut frontend = Frontend::default();
    let options = Options::from(stage);
    let Ok(res) = frontend.parse(&options, glsl) else { panic!("Failed to parse shader") };
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let Ok(module_info) = validator.validate(&res) else { panic!("Failed to validate shader") };
    let code = wgsl::write_string(&res, &module_info, wgsl::WriterFlags::all()).unwrap();
    code
}

// wgsl to msl
use naga::back::msl;
use naga::back::msl::TranslationInfo;

pub fn wgsl_to_msl(wgsl: &str) -> (String, TranslationInfo) {
    let module = naga::front::wgsl::parse_str(wgsl).unwrap();
    let info = Validator::new(ValidationFlags::all(), Capabilities::empty()).validate(&module).unwrap();
    let options = msl::Options {
        lang_version: (2, 1),
        ..Default::default()
    };
    let code = msl::write_string(&module, &info, &options, &Default::default()).unwrap();
    code
}