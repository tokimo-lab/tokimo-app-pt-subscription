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
        while let Some(pair) = dict.next_pair() {
            match pair? {
                (b"info", value) => {
                    let info = value.try_into_dictionary()?;
                    let mut info_name = String::new();
                    let mut file_list: Vec<(String, u64)> = Vec::new();

                    let mut info_dict = info;
                    while let Some(ip) = info_dict.next_pair() {
                        match ip? {
                            (b"name", v) => {
                                info_name = String::decode_bencode_object(v)
                                    .unwrap_or_default();
                            }
                            (b"files", v) => {
                                let mut file_dicts = v.try_into_list()?;
                                while let Some(file_obj) = file_dicts.next_object() {
                                    let file_dict = file_obj?.try_into_dictionary()?;
                                    let mut path_parts: Vec<String> = Vec::new();
                                    let mut size: u64 = 0;

                                    let mut fd = file_dict;
                                    while let Some(fp) = fd.next_pair() {
                                        match fp? {
                                            (b"path", pv) => {
                                                let mut parts_list = pv.try_into_list()?;
                                                while let Some(part) = parts_list.next_object() {
                                                    let s = String::decode_bencode_object(part?)?;
                                                    path_parts.push(s);
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
                                // Single-file torrent
                                let size = u64::decode_bencode_object(v)?;
                                file_list.push((String::new(), size));
                            }
                            _ => {}
                        }
                    }

                    name = info_name;

                    if file_list.is_empty() {
                        // Single file with name only
                        files.push(TorrentFileInfo {
                            index: 0,
                            path: name.clone(),
                            size: 0,
                        });
                    } else if file_list.len() == 1 && file_list[0].0.is_empty() {
                        files.push(TorrentFileInfo {
                            index: 0,
                            path: name.clone(),
                            size: file_list[0].1,
                        });
                    } else {
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
                    }
                }
                _ => {}
            }
        }

        Ok(TorrentMeta { name, files })
    }
}

pub fn parse_torrent(data: &[u8]) -> Result<TorrentMeta, String> {
    TorrentMeta::from_bencode(data).map_err(|e| format!("Failed to parse torrent: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_file() {
        // A minimal single-file torrent (bencoded)
        // d8:announce3:url4:infod4:name8:test.mkv6:lengthi1024eee
        let data = b"d8:announce3:url4:infod4:name8:test.mkv6:lengthi1024eee";
        let meta = parse_torrent(data).unwrap();
        assert_eq!(meta.name, "test.mkv");
        assert_eq!(meta.files.len(), 1);
        assert_eq!(meta.files[0].path, "test.mkv");
        assert_eq!(meta.files[0].size, 1024);
    }
}
