use naga::{
    back::spv::Options,
    valid::{Capabilities, ValidationFlags, Validator},
};
use proc_macro::TokenStream;

#[proc_macro]
pub fn wgsl_compile(input: TokenStream) -> TokenStream {
    // Just grab the raw input
    let shader_code = input.to_string();

    let module = match naga::front::wgsl::parse_str(&shader_code) {
        Ok(module) => module,
        Err(err) => panic!("WGSL parsing error: {}", err.emit_to_string(&shader_code)),
    };

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let module_info = match validator.validate(&module) {
        Ok(info) => info,
        Err(err) => {
            panic!(
                "WGSL validation error: {}",
                err.emit_to_string(&shader_code)
            );
        }
    };

    let mut output_buffer = Vec::new();
    let mut spirv_writer =
        naga::back::spv::Writer::new(&Options::default()).expect("Failed to create SPIR-V writer");

    spirv_writer
        .write(&module, &module_info, None, &None, &mut output_buffer)
        .expect("Failed to write SPIR-V");

    let output = quote::quote! {
        &[#(#output_buffer),*]
    };

    output.into()
}
