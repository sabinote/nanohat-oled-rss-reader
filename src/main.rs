mod nanohat;
mod rss;

#[tokio::main]
async fn main() {
    let xml = reqwest::get("https://news.yahoo.co.jp/rss/topics/top-picks.xml")
        .await?
        .test()
        .await?;

    let rss = rss::RSS::from_str(s)?;
    println!("{}", rss.channel.title);
}
