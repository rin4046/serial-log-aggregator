use anyhow::Result;
use chrono::Local;
use clap::Parser;
use once_cell::sync::Lazy;
use std::net::TcpStream;
use std::sync::Mutex;
use std::{fs, io::Write};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{client, WebSocket};

mod serial_per_line;
use serial_per_line::SerialPerLine;

#[derive(Parser, Debug)]
#[clap()]
struct Args {
    #[clap(short, long, value_parser)]
    port: String,
    #[clap(short, long, value_parser)]
    baud: u32,
    #[clap(short, long, value_parser)]
    out: String,
    #[clap(long, value_parser)]
    begin: String,
    #[clap(long, value_parser)]
    end: String,
}

static ARGS: Lazy<Args> = Lazy::new(|| Args::parse());

static SOCKET: Lazy<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>> = Lazy::new(|| {
    let (socket, _) = client::connect("ws://localhost:5000/rx").unwrap();
    Mutex::new(socket)
});

fn main() -> Result<()> {
    let port = serialport::new(ARGS.port.clone(), ARGS.baud)
        .timeout(std::time::Duration::from_millis(10))
        .open()?;

    let mut serial_per_line = SerialPerLine::new(port, process_line);

    loop {
        serial_per_line.read()?;
    }
}

fn process_line(line: &String, file: &mut Option<fs::File>) -> Result<()> {
    println!("{}", *line);

    if *line == ARGS.begin {
        // ディレクトリ作成
        let dir_path = format!("{}/{}", ARGS.out, Local::now().format("%Y_%m_%d"));
        // ファイル作成
        fs::create_dir_all(&dir_path)?;
        let file_path = format!("{}/{}.csv", dir_path, Local::now().format("%H_%M_%S"));
        *file = Some(fs::File::create(&file_path)?);
        println!("{}", file_path);
    } else if *line == ARGS.end {
        // ファイルを閉じる
        *file = None;
        println!("ログ出力を停止しました")
    } else {
        // ファイルに追記
        if let Some(file) = &mut *file {
            file.write_all(line.as_bytes())?;
            file.write_all(b"\n")?;
        }
    }

    // リアルタイムで
    if let Ok(_) = SOCKET
        .lock()
        .unwrap()
        .write_message(tungstenite::Message::Text(line.clone()))
    {}

    Ok(())
}
