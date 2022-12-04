#[macro_export]
macro_rules! create_spv_shader {
    ($device: expr, $file: expr, $label: expr) => {
        {
            log::info!("Compiling {} into module named {}.", $file, $label);

            let source = include_bytes!($file).to_vec();
            // let mut rdr: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(source.clone());
            let mut bytes: Vec<u32> = vec![];

            for i in (0..source.len()).step_by(4) {
                // bytes.push(rdr.read_u32::<byteorder::LittleEndian>().unwrap_or(0));
                bytes.push(u32::from_ne_bytes([
                                              *source.get(i).unwrap(),
                                              *source.get(i+1).unwrap(),
                                              *source.get(i+2).unwrap(),
                                              *source.get(i+3).unwrap(),
                ]));
            }

            $device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some($label),
                // Probably move this exclusively to compile time in the future.
                source: wgpu::ShaderSource::SpirV(bytes.into()),
            })
        }
    }
}

pub(crate) use create_spv_shader;

