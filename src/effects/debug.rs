use crate::*;
use num_derive::FromPrimitive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
enum Type {POSITION = 0, NORMAL = 1, COLOR = 2, DEPTH = 3, DIFFUSE = 4, SPECULAR = 5, POWER = 6, NONE = 7}

pub struct DebugEffect {
    gl: Gl,
    program: program::Program,
    debug_type: Type,
    buffer: VertexBuffer
}

impl DebugEffect {

    pub fn new(gl: &Gl) -> Result<DebugEffect, effects::Error>
    {
        let program = program::Program::from_source(&gl,
                                                    include_str!("shaders/effect.vert"),
                                                    include_str!("shaders/debug.frag"))?;

        let positions = vec![
            -3.0, -1.0, 0.0,
            3.0, -1.0, 0.0,
            0.0, 2.0, 0.0
        ];
        let uvs = vec![
            -1.0, 0.0,
            2.0, 0.0,
            0.5, 1.5
        ];
        let buffer = VertexBuffer::new_with_two_static_attributes(&gl, &positions, &uvs).unwrap();

        Ok(DebugEffect {gl: gl.clone(), program, debug_type: Type::NONE, buffer})
    }

    pub fn change_type(&mut self)
    {
        self.debug_type = num::FromPrimitive::from_u32(((self.debug_type as u32) + 1) % (Type::NONE as u32 + 1)).unwrap();
        println!("{:?}", self.debug_type);
    }

    pub fn apply(&self, camera: &Camera, geometry_texture: &dyn Texture, depth_texture: &dyn Texture) -> Result<(), effects::Error>
    {
        if self.debug_type != Type::NONE {
            state::depth_write(&self.gl,false);
            state::depth_test(&self.gl, state::DepthTestType::None);
            state::cull(&self.gl,state::CullType::Back);
            state::blend(&self.gl, state::BlendType::None);

            self.program.add_uniform_mat4("viewProjectionInverse", &(camera.get_projection() * camera.get_view()).invert().unwrap())?;

            geometry_texture.bind(0);
            self.program.add_uniform_int("gbuffer", &0)?;

            depth_texture.bind(1);
            self.program.add_uniform_int("depthMap", &1)?;

            self.program.add_uniform_int("type", &(self.debug_type as i32))?;

            self.program.use_attribute_vec3_float(&self.buffer, "position", 0).unwrap();
            self.program.use_attribute_vec2_float(&self.buffer, "uv_coordinate", 1).unwrap();
            self.program.draw_arrays(3);
        }
        Ok(())
    }

}