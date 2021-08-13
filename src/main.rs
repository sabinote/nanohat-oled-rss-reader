mod nanohat;
mod rss;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let xml = reqwest::get("https://news.yahoo.co.jp/rss/topics/top-picks.xml")
        .await?
        .text()
        .await?;

    let rss = rss::RSS::new(&xml)?;
    println!("{}", rss.channel.title);
    Ok(())
}
