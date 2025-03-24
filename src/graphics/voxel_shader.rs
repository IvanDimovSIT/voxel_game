use macroquad::prelude::{gl_use_material, load_material, Comparison, Material, MaterialParams, PipelineParams, ShaderSource};


pub const VERTEX_SHADER: &str = "
#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
";

pub const FRAGMENT_SHADER: &str = "
#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv);
}
";

pub struct VoxelShader {
    material: Material
}
impl VoxelShader {
    pub fn new() -> Self {
        let pipeline_params = PipelineParams {
            depth_write: true,
            depth_test: Comparison::LessOrEqual,
            cull_face: macroquad::miniquad::CullFace::Back,
            ..Default::default()
        };
    
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params,
                ..Default::default()
                
            },
        )
        .expect("Error initialising shaders");

        Self { material }
    }

    pub fn set_material(&self) {
        gl_use_material(&self.material);
    }
}