use bendy::decoding::FromBencode;

#[derive(Debug, Clone)]
pub struct TorrentFileInfo {
    pub index: usize,
    pub path: String,
    pub size: u64,
}

#[derive(Debug)]
pub struct TorrentMeta {
    pub name: String,
    pub files: Vec<TorrentFileInfo>,
}

impl FromBencode for TorrentMeta {
    fn decode_bencode_object(object: bendy::decoding::Object) -> Result<Self, bendy::decoding::Error> {
        let mut name = String::new();
        let mut files = Vec::new();

        let mut dict = object.try_into_dictionary()?;
        while let Ok(Some(pair)) = dict.next_pair() {
            if let (b"info", value) = pair {
                let mut info = value.try_into_dictionary()?;
                let mut info_name = String::new();
                let mut file_list: Vec<(String, u64)> = Vec::new();
                let mut single_length: Option<u64> = None;

                while let Ok(Some(ip)) = info.next_pair() {
                    match ip {
                        (b"name", v) => {
                            info_name = String::decode_bencode_object(v).unwrap_or_default();
                        }
                        (b"files", v) => {
                            let mut file_dicts = v.try_into_list()?;
                            while let Ok(Some(file_obj)) = file_dicts.next_object() {
                                let mut fd = file_obj.try_into_dictionary()?;
                                let mut path_parts: Vec<String> = Vec::new();
                                let mut size: u64 = 0;

                                while let Ok(Some(fp)) = fd.next_pair() {
                                    match fp {
                                        (b"path", pv) => {
                                            let mut parts_list = pv.try_into_list()?;
                                            while let Ok(Some(part)) = parts_list.next_object() {
                                                if let Ok(s) = String::decode_bencode_object(part) {
                                                    path_parts.push(s);
                                                }
                                            }
                                        }
                                        (b"length", lv) => {
                                            size = u64::decode_bencode_object(lv)?;
                                        }
                                        _ => {}
                                    }
                                }
                                let path = path_parts.join("/");
                                if !path.is_empty() {
                                    file_list.push((path, size));
                                }
                            }
                        }
                        (b"length", v) => {
                            single_length = Some(u64::decode_bencode_object(v)?);
                        }
                        _ => {}
                    }
                }

                name = info_name;

                if !file_list.is_empty() {
                    for (i, (path, size)) in file_list.into_iter().enumerate() {
                        files.push(TorrentFileInfo {
                            index: i,
                            path: if name.is_empty() {
                                path
                            } else {
                                format!("{name}/{path}")
                            },
                            size,
                        });
                    }
                } else if let Some(size) = single_length {
                    files.push(TorrentFileInfo {
                        index: 0,
                        path: name.clone(),
                        size,
                    });
                }
            }
        }

        Ok(TorrentMeta { name, files })
    }
}

pub fn parse_torrent(data: &[u8]) -> Result<TorrentMeta, String> {
    TorrentMeta::from_bencode(data).map_err(|e| format!("Failed to parse torrent: {e}"))
}
