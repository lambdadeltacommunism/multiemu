struct EguiVertex {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> egui_vertex_buffer: array<EguiVertex>;
@group(0) @binding(1) var<storage, read> texture_sampler: sampler;

@group(1) @binding(0) var<storage, read> screen_size: vec2<f32>;

@vertex
fn vertex_main(vertex_input: EguiVertex) -> VertexOutput {
    let vertex_output = VertexOutput(
        vec4(
            2.0 * vertex_input.pos.x / screen_size.x - 1.0,
            2.0 * vertex_input.pos.y / screen_size.y - 1.0,
            0.0,
            1.0,
        ),
        vertex_input.uv,
        vertex_input.color,
    );

    return vertex_output;
}

@fragment
fn fragment_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(texture_sampler, vertex_output.uv);
    let color = linear_to_srgba(texture_color) * vertex_output.color;
    return color;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(c / 12.92, pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c > vec3<f32>(0.04045));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(c * 12.92, pow(c, vec3<f32>(1.0 / 2.4)) * 1.055 - 0.055, c < vec3<f32>(0.0031308));
}

fn srgba_to_linear(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(srgb_to_linear(c.rgb), c.a);
}

fn linear_to_srgba(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(linear_to_srgb(c.rgb), c.a);
}
