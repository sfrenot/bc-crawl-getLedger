pub struct PublicKey {
    bytes: Vec<u8>
}

impl From<Vec<u8>> for PublicKey {
    fn from(bytes: Vec<u8>) -> Self {
        let len;
        if bytes.is_empty() {
            len = 0;
        } else {
            len = get_len(bytes[0]);
        }

        if len != 0 && len == bytes.len() {
            PublicKey { bytes }
        } else {
            PublicKey { bytes: Vec::from([0xff]) }
        }
    }
}

impl PublicKey {
    pub fn len(&self) -> usize {
        get_len(self.bytes[0])
    }

    pub fn is_valid(&self) -> bool {
        self.len() != 0
    }
}

fn get_len(first_byte: u8) -> usize {
    if first_byte == 2 || first_byte == 3 {
        33
    } else if first_byte == 4 || first_byte == 6 || first_byte == 7 {
        65
    } else {
        0
    }
}