use anyhow::{Result, bail};
use glam::{Vec2, Vec3A};
use std::io::{Cursor, Read};

pub(crate) struct Reader {
    cur: Cursor<Vec<u8>>,
    pub encoding: u8,
    pub additional_uv: u8,
    pub vertex_index_size: u8,
    pub texture_index_size: u8,
    pub material_index_size: u8,
}

impl Reader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            cur: Cursor::new(data),
            encoding: 0,
            additional_uv: 0,
            vertex_index_size: 4,
            texture_index_size: 4,
            material_index_size: 4,
        }
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut b = [0u8; 1];
        self.cur.read_exact(&mut b)?;
        Ok(b[0])
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        let mut b = [0u8; 1];
        self.cur.read_exact(&mut b)?;
        Ok(b[0] as i8)
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let mut b = [0u8; 2];
        self.cur.read_exact(&mut b)?;
        Ok(i16::from_le_bytes(b))
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut b = [0u8; 2];
        self.cur.read_exact(&mut b)?;
        Ok(u16::from_le_bytes(b))
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let mut b = [0u8; 4];
        self.cur.read_exact(&mut b)?;
        Ok(i32::from_le_bytes(b))
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        let mut b = [0u8; 4];
        self.cur.read_exact(&mut b)?;
        Ok(f32::from_le_bytes(b))
    }

    pub fn read_vec3(&mut self) -> Result<Vec3A> {
        Ok(Vec3A::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    pub fn read_vec2(&mut self) -> Result<Vec2> {
        Ok(Vec2::new(self.read_f32()?, self.read_f32()?))
    }

    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.cur.read_exact(buf)?;
        Ok(())
    }

    pub fn read_string(&mut self) -> Result<String> {
        let len = self.read_i32()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        let mut bytes = vec![0u8; len];
        self.cur.read_exact(&mut bytes)?;
        if self.encoding == 0 {
            let u16s: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            Ok(String::from_utf16(&u16s)?)
        } else {
            Ok(String::from_utf8(bytes)?)
        }
    }

    pub fn read_index(&mut self, size: u8) -> Result<i32> {
        match size {
            1 => Ok(self.read_i8()? as i32),
            2 => Ok(self.read_i16()? as i32),
            4 => Ok(self.read_i32()?),
            _ => bail!("invalid index size: {size}"),
        }
    }

    pub fn read_raw_vertex_index(&mut self) -> Result<i32> {
        match self.vertex_index_size {
            1 => Ok(self.read_u8()? as i32),
            2 => Ok(self.read_u16()? as i32),
            4 => {
                let v = self.read_i32()?;
                if v < 0 {
                    bail!("negative vertex index: {v}");
                }
                Ok(v)
            }
            _ => bail!("invalid vertex index size: {}", self.vertex_index_size),
        }
    }

    pub fn read_texture_index(&mut self) -> Result<Option<usize>> {
        let idx = self.read_index(self.texture_index_size)?;
        if idx < 0 {
            Ok(None)
        } else {
            Ok(Some(idx as usize))
        }
    }
}

pub(crate) fn read_header(r: &mut Reader) -> Result<()> {
    let mut magic = [0u8; 4];
    r.read_bytes(&mut magic)?;
    if &magic != b"PMX " {
        bail!("not a valid PMX file");
    }
    let version = r.read_f32()?;
    if !(2.0..=2.2).contains(&version) {
        bail!("unsupported PMX version: {version}");
    }
    let flags_len = r.read_u8()?;
    if flags_len < 8 {
        bail!("invalid PMX header length");
    }
    r.encoding = r.read_u8()?;
    r.additional_uv = r.read_u8()?;
    r.vertex_index_size = r.read_u8()?;
    r.texture_index_size = r.read_u8()?;
    r.material_index_size = r.read_u8()?;
    let _bone_index_size = r.read_u8()?;
    let _morph_index_size = r.read_u8()?;
    let _rigid_body_index_size = r.read_u8()?;
    for _ in 8..flags_len {
        r.read_u8()?;
    }
    Ok(())
}

pub(crate) fn skip_model_info(r: &mut Reader) -> Result<()> {
    r.read_string()?;
    r.read_string()?;
    r.read_string()?;
    r.read_string()?;
    Ok(())
}
