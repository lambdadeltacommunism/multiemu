use naga::valid::{Capabilities, ValidationFlags, Validator};
use vulkano::pipeline::graphics::vertex_input::{
    VertexInputAttributeDescription, VertexInputState,
};

pub struct VulkanShader {
    pub vertex_entry_point: String,
    pub fragment_entry_point: String,
    pub spirv: Vec<u32>,
    pub vertex_input_state: VertexInputState,
}

pub fn compile_shader(source: &str) -> VulkanShader {
    let parsed_shader = naga::front::wgsl::parse_str(source).unwrap();
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let parsed_shader_info = validator.validate(&parsed_shader).unwrap();

    let mut vertex_entry_point = None;
    let mut fragment_entry_point = None;

    for entry_point in parsed_shader.entry_points {
        match entry_point.stage {
            naga::ShaderStage::Vertex => {
                vertex_entry_point = Some(entry_point.name);
            }
            naga::ShaderStage::Fragment => {
                fragment_entry_point = Some(entry_point.name);
            }
            naga::ShaderStage::Compute => todo!(),
        }
    }

    let vertex_input_state = VertexInputState::new().attribute(
        0,
        VertexInputAttributeDescription {
            binding: todo!(),
            format: todo!(),
            offset: todo!(),
        },
    );

    VulkanShader {
        vertex_entry_point: vertex_entry_point.unwrap(),
        fragment_entry_point: fragment_entry_point.unwrap(),
        spirv: naga::back::spv::write_vec(
            &parsed_shader,
            &parsed_shader_info,
            &naga::back::spv::Options::default(),
            None,
        )
        .unwrap(),
        vertex_input_state,
    }
}
