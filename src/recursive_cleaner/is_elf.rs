use std::io::Read;

pub enum IsElfError {
    Io {
        kind: std::io::ErrorKind,
        when: &'static str,
    },
}
pub fn is_elf<T: std::convert::AsRef<std::path::Path>>(file_path: T) -> Result<bool, IsElfError> {
    let mut file = match std::fs::File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            return Err(IsElfError::Io {
                kind: err.kind(),
                when: "opening file",
            });
        }
    };

    let mut start_bytes: [u8; 4] = [0; 4];
    match file.read_exact(&mut start_bytes) {
        Ok(_) => {}
        Err(err) => {
            return Err(IsElfError::Io {
                kind: err.kind(),
                when: "reading on file",
            });
        }
    };
    if start_bytes.eq(&[0x7F, b'E', b'L', b'F']) {
        Ok(true)
    } else {
        Ok(false)
    }
}
