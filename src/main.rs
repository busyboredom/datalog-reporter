use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use printpdf::*;

use std::fs::File;
use std::io::BufWriter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = Config::new();

    config.host("192.168.0.100");
    config.port(1433);
    config.database("Datalogging");
    config.encryption(tiberius::EncryptionLevel::On);
    config.trust_cert();
    config.authentication(AuthMethod::sql_server("SA", "20-570"));

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    // To be able to use Tokio's tcp, we're using the `compat_write` from
    // the `TokioAsyncWriteCompatExt` to get a stream compatible with the
    // traits from the `futures` crate.
    print!("Connecting...");
    let mut client = Client::connect(config, tcp.compat_write()).await?;
    println!("Done");

    // Create blank PDF.
    let (doc, page1, layer1) = PdfDocument::new("Batch Report", Mm(247.0), Mm(210.0), "Layer 1");
    let (page2, layer1) = doc.add_page(Mm(247.0), Mm(210.0),"Page 2, Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_builtin_font(printpdf::BuiltinFont::TimesRoman).expect("Unabele to add TimesRoman font");

    // Get data.
    let mut stream = client.query("SELECT TOP (@P1) * FROM FloatTable", &[& 1i32]).await?;
     
    // Process data.
    let row = stream.into_row().await?;
    let val: f64 = row.unwrap().get("Val").unwrap();
    let text = format!("Data from database: {}", val);

    current_layer.use_text(text, 11.0, Mm(10.0), Mm(200.0), &font);

    // Save PDF.
    doc.save(&mut BufWriter::new(File::create("test_working.pdf").unwrap())).unwrap();

    Ok(())
}
