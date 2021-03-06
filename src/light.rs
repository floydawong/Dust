
use crate::*;

#[derive(Debug)]
pub enum Error {
    Buffer(buffer::Error),
    Rendertarget(rendertarget::Error)
}

impl From<buffer::Error> for Error {
    fn from(other: buffer::Error) -> Self {
        Error::Buffer(other)
    }
}

impl From<rendertarget::Error> for Error {
    fn from(other: rendertarget::Error) -> Self {
        Error::Rendertarget(other)
    }
}

pub const MAX_NO_LIGHTS: usize = 4;

pub struct AmbientLight
{
    color: Vec3,
    intensity: f32
}

impl AmbientLight
{
    pub(crate) fn new() -> AmbientLight
    {
        AmbientLight { color: vec3(1.0, 1.0, 1.0), intensity: 0.5 }
    }

    pub fn color(&self) -> Vec3
    {
        self.color
    }

    pub fn set_color(&mut self, color: &Vec3)
    {
        self.color = *color;
    }

    pub fn intensity(&self) -> f32
    {
        self.intensity
    }

    pub fn set_intensity(&mut self, intensity: f32)
    {
        self.intensity = intensity;
    }
}

pub struct DirectionalLight {
    gl: Gl,
    light_buffer: UniformBuffer,
    shadow_texture: Texture2DArray,
    shadow_rendertarget: RenderTarget,
    shadow_cameras: [Option<Camera>; MAX_NO_LIGHTS],
    index: usize
}

impl DirectionalLight {

    pub(crate) fn new(gl: &Gl) -> Result<DirectionalLight, Error>
    {
        let uniform_sizes: Vec<u32> = [3u32, 1, 3, 1, 16].iter().cloned().cycle().take(5*MAX_NO_LIGHTS).collect();

        let shadow_texture = Texture2DArray::new_as_depth_targets(gl, 1024, 1024, MAX_NO_LIGHTS).unwrap();
        let mut lights = DirectionalLight {
            gl: gl.clone(),
            shadow_texture,
            shadow_rendertarget: RenderTarget::new(gl, 0)?,
            light_buffer: UniformBuffer::new(gl, &uniform_sizes)?,
            shadow_cameras: [None, None, None, None],
            index: 0};

        for light_id in 0..MAX_NO_LIGHTS {
            let light = lights.light_at(light_id);
            light.set_intensity(0.0);
            light.set_color(&vec3(1.0, 1.0, 1.0));
            light.set_direction(&vec3(0.0, -1.0, 0.0));
            light.disable_shadows();
        }
        Ok(lights)
    }

    pub fn set_color(&mut self, color: &Vec3)
    {
        self.light_buffer.update(self.index_at(0), &color.to_slice()).unwrap();
    }

    pub fn set_intensity(&mut self, intensity: f32)
    {
        self.light_buffer.update(self.index_at(1), &[intensity]).unwrap();
    }

    pub fn set_direction(&mut self, direction: &Vec3)
    {
        self.light_buffer.update(self.index_at(2), &direction.to_slice()).unwrap();
        self.update_shadows(vec3(0.0, 0.0, 0.0), 4.0, 20.0);
    }

    pub fn direction(&self) -> Vec3 {
        let d = self.light_buffer.get(self.index_at(2)).unwrap();
        vec3(d[0], d[1], d[2])
    }

    pub fn is_shadows_enabled(&self) -> bool {
        self.shadow_cameras[self.index].is_some()
    }

    pub fn enable_shadows(&mut self)
    {
        self.update_shadows(vec3(0.0, 0.0, 0.0), 4.0, 20.0);
    }

    pub fn update_shadows(&mut self, target: Vec3, size: f32, depth: f32) {
        let direction = self.direction();
        let up = compute_up_direction(direction);

        if let Some(ref mut camera) = self.shadow_cameras[self.index]
        {
            camera.set_view(target - direction, target, up);
            camera.set_orthographic_projection(size, size, depth);
        }
        else {
            let camera = Camera::new_orthographic(&self.gl, target - direction, target, up, size, size, depth);
            self.shadow_cameras[self.index] = Some(camera);
        }

        let camera = self.shadow_cameras[self.index].as_ref().unwrap();
        self.light_buffer.update(self.index_at(4), &shadow_matrix(camera).to_slice()).unwrap();
    }

    pub fn disable_shadows(&mut self)
    {
        self.shadow_cameras[self.index] = None;
        self.light_buffer.update(self.index_at(4), &Mat4::from_value(0.0).to_slice()).unwrap();
    }

    pub(crate) fn shadow_pass<F>(&self, render_scene: &F)
        where F: Fn(&Camera)
    {
        for light_id in 0..MAX_NO_LIGHTS {
            if let Some(ref camera) = self.shadow_cameras[light_id]
            {
                self.shadow_rendertarget.write_to_depth_array(&self.shadow_texture, light_id).unwrap();
                self.shadow_rendertarget.clear_depth();
                render_scene(camera);
            }
        }
    }

    pub(crate) fn shadow_maps(&self) -> &Texture2DArray
    {
        &self.shadow_texture
    }

    pub(crate) fn buffer(&self) -> &UniformBuffer
    {
        &self.light_buffer
    }

    pub(crate) fn light_at(&mut self, index: usize) -> &mut Self
    {
        self.index = index;
        self
    }

    fn index_at(&self, index: usize) -> usize
    {
        self.index * 5 + index
    }
}

pub struct PointLight {
    light_buffer: UniformBuffer,
    index: usize
}

impl PointLight {

    pub(crate) fn new(gl: &Gl) -> Result<PointLight, Error>
    {
        let uniform_sizes: Vec<u32> = [3u32, 1, 1, 1, 1, 1, 3, 1].iter().cloned().cycle().take(8*MAX_NO_LIGHTS).collect();
        let mut lights = PointLight {
            light_buffer: UniformBuffer::new(gl, &uniform_sizes)?,
            index: 0};

        for light_id in 0..MAX_NO_LIGHTS {
            let light = lights.light_at(light_id);
            light.set_intensity(0.0);
            light.set_color(&vec3(1.0, 1.0, 1.0));
            light.set_position(&vec3(0.0, 0.0, 0.0));
            light.set_attenuation(0.5, 0.05, 0.005);
        }
        Ok(lights)
    }

    pub fn set_color(&mut self, color: &Vec3)
    {
        self.light_buffer.update(self.index_at(0), &color.to_slice()).unwrap();
    }

    pub fn set_intensity(&mut self, intensity: f32)
    {
        self.light_buffer.update(self.index_at(1), &[intensity]).unwrap();
    }

    pub fn set_attenuation(&mut self, constant: f32, linear: f32, exponential: f32)
    {
        self.light_buffer.update(self.index_at(2), &[constant]).unwrap();
        self.light_buffer.update(self.index_at(3), &[linear]).unwrap();
        self.light_buffer.update(self.index_at(4), &[exponential]).unwrap();
    }

    pub fn set_position(&mut self, position: &Vec3)
    {
        self.light_buffer.update(self.index_at(6), &position.to_slice()).unwrap();
    }

    pub(crate) fn buffer(&self) -> &UniformBuffer
    {
        &self.light_buffer
    }

    pub(crate) fn light_at(&mut self, index: usize) -> &mut Self
    {
        self.index = index;
        self
    }

    fn index_at(&self, index: usize) -> usize
    {
        self.index * 8 + index
    }
}

pub struct SpotLight {
    gl: Gl,
    light_buffer: UniformBuffer,
    shadow_texture: Texture2DArray,
    shadow_rendertarget: RenderTarget,
    shadow_cameras: [Option<Camera>; MAX_NO_LIGHTS],
    index: usize
}

impl SpotLight {

    pub(crate) fn new(gl: &Gl) -> Result<SpotLight, Error>
    {
        let uniform_sizes: Vec<u32> = [3u32, 1, 1, 1, 1, 1, 3, 1, 3, 1, 16].iter().cloned().cycle().take(11*MAX_NO_LIGHTS).collect();
        let shadow_texture = Texture2DArray::new_as_depth_targets(gl, 1024, 1024, MAX_NO_LIGHTS).unwrap();
        let mut lights = SpotLight {
            gl: gl.clone(),
            shadow_texture,
            shadow_rendertarget: RenderTarget::new(gl, 0)?,
            light_buffer: UniformBuffer::new(gl, &uniform_sizes)?,
            shadow_cameras: [None, None, None, None],
            index: 0};

        for light_id in 0..MAX_NO_LIGHTS {
            let light = lights.light_at(light_id);
            light.set_intensity(0.0);
            light.set_color(&vec3(1.0, 1.0, 1.0));
            light.set_cutoff(0.1 * std::f32::consts::PI);
            light.set_direction(&vec3(0.0, -1.0, 0.0));
            light.set_position(&vec3(0.0, 0.0, 0.0));
            light.set_attenuation(0.5, 0.05, 0.005);
            light.disable_shadows();
        }
        Ok(lights)
    }

    pub fn set_color(&mut self, color: &Vec3)
    {
        self.light_buffer.update(self.index_at(0), &color.to_slice()).unwrap();
    }

    pub fn set_intensity(&mut self, intensity: f32)
    {
        self.light_buffer.update(self.index_at(1), &[intensity]).unwrap();
    }

    pub fn set_attenuation(&mut self, constant: f32, linear: f32, exponential: f32)
    {
        self.light_buffer.update(self.index_at(2), &[constant]).unwrap();
        self.light_buffer.update(self.index_at(3), &[linear]).unwrap();
        self.light_buffer.update(self.index_at(4), &[exponential]).unwrap();
    }

    pub fn set_position(&mut self, position: &Vec3)
    {
        self.light_buffer.update(self.index_at(6), &position.to_slice()).unwrap();
        self.update_shadow_camera();
    }

    pub fn position(&self) -> Vec3
    {
        let p = self.light_buffer.get(self.index_at(6)).unwrap();
        vec3(p[0], p[1], p[2])
    }

    pub fn set_cutoff(&mut self, cutoff: f32)
    {
        self.light_buffer.update(self.index_at(7), &[cutoff]).unwrap();
        self.update_shadow_camera();
    }

    pub fn set_direction(&mut self, direction: &Vec3)
    {
        self.light_buffer.update(self.index_at(8), &direction.normalize().to_slice()).unwrap();
        self.update_shadow_camera();
    }

    pub fn direction(&self) -> Vec3
    {
        let d = self.light_buffer.get(self.index_at(8)).unwrap();
        vec3(d[0], d[1], d[2])
    }

    fn update_shadow_camera(&mut self)
    {
        let position = self.position();
        let direction = self.direction();
        let up = compute_up_direction(direction);

        let depth = 200.0;
        let cutoff = self.light_buffer.get(self.index_at(7)).unwrap()[0];

        if let Some(ref mut camera) = self.shadow_cameras[self.index]
        {
            camera.set_view(position, position + direction, up);
            camera.set_perspective_projection(degrees(45.0), 2.0 * cutoff, 0.1, depth);
        }
        else {
            let camera = Camera::new_perspective(&self.gl, position, position + direction, up, degrees(45.0), 2.0 * cutoff, 0.1, depth);
            self.shadow_cameras[self.index] = Some(camera);
        }

        let shadow_matrix = shadow_matrix(self.shadow_cameras[self.index].as_ref().unwrap());
        self.light_buffer.update(self.index_at(10), &shadow_matrix.to_slice()).unwrap();
    }

    pub fn is_shadows_enabled(&self) -> bool {
        self.shadow_cameras[self.index].is_some()
    }

    pub fn enable_shadows(&mut self)
    {
        self.update_shadow_camera();
    }

    pub fn disable_shadows(&mut self)
    {
        self.shadow_cameras[self.index] = None;
        self.light_buffer.update(self.index_at(10), &Mat4::from_value(0.0).to_slice()).unwrap();
    }

    pub(crate) fn shadow_pass<F>(&self, render_scene: &F)
        where F: Fn(&Camera)
    {
        for light_id in 0..MAX_NO_LIGHTS {
            if let Some(ref camera) = self.shadow_cameras[light_id]
            {
                self.shadow_rendertarget.write_to_depth_array(&self.shadow_texture, light_id).unwrap();
                self.shadow_rendertarget.clear_depth();
                render_scene(camera);
            }
        }
    }

    pub(crate) fn shadow_maps(&self) -> &Texture2DArray
    {
        &self.shadow_texture
    }

    pub(crate) fn buffer(&self) -> &UniformBuffer
    {
        &self.light_buffer
    }

    pub(crate) fn light_at(&mut self, index: usize) -> &mut Self
    {
        self.index = index;
        self
    }

    fn index_at(&self, index: usize) -> usize
    {
        self.index * 11 + index
    }
}

fn shadow_matrix(camera: &Camera) -> Mat4
{
    let bias_matrix = crate::Mat4::new(
                         0.5, 0.0, 0.0, 0.0,
                         0.0, 0.5, 0.0, 0.0,
                         0.0, 0.0, 0.5, 0.0,
                         0.5, 0.5, 0.5, 1.0);
    bias_matrix * camera.get_projection() * camera.get_view()
}

fn compute_up_direction(direction: Vec3) -> Vec3
{
    if vec3(1.0, 0.0, 0.0).dot(direction).abs() > 0.9
    {
        (vec3(0.0, 1.0, 0.0).cross(direction)).normalize()
    }
    else {
        (vec3(1.0, 0.0, 0.0).cross(direction)).normalize()
    }
}