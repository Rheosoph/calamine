use calamine::{Ods, Reader, Xlsb, Xlsx};
use std::io::{Cursor, Read};

#[test]
fn workbook_readers_reject_conflicting_zip_header_names() {
    for (format, bytes) in [
        ("xlsx", include_bytes!("issues.xlsx").as_slice()),
        ("xlsb", include_bytes!("issues.xlsb").as_slice()),
        ("ods", include_bytes!("issues.ods").as_slice()),
    ] {
        let read = |data: Vec<u8>| -> Result<(), String> {
            match format {
                "xlsx" => Xlsx::new(Cursor::new(data))
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                "xlsb" => Xlsb::new(Cursor::new(data))
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                "ods" => Ods::new(Cursor::new(data))
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                _ => unreachable!(),
            }
        };
        read(bytes.to_vec()).unwrap();
        let (offset, name) = {
            let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
            let entry = archive.by_index(0).unwrap();
            (
                entry.header_start() as usize + 30,
                entry.name().unwrap().into_owned(),
            )
        };
        let mut cloaked = bytes.to_vec();
        assert_eq!(&cloaked[offset..offset + name.len()], name.as_bytes());
        cloaked[offset] = if cloaked[offset] == b'x' { b'y' } else { b'x' };
        let mut archive = zip::ZipArchive::new(Cursor::new(&cloaked)).unwrap();
        let mut content = Vec::new();
        archive
            .by_name(&name)
            .unwrap()
            .read_to_end(&mut content)
            .unwrap();
        let error = read(cloaked).expect_err("conflicting names must be rejected");
        assert!(error.contains("filename mismatch"), "{format}: {error}");
    }
}
