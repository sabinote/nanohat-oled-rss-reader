use serde::Deserialize;
use std::str::FromStr;
use serde_xml_rs::Error;

#[derive(Deserialize)]
#[serde(rename = "rss")]
pub struct RSS {
    pub channel: Channel,
}

impl FromStr for RSS {
    type Err = Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_xml_rs::from_str(s)
    }
}

#[derive(Deserialize)]
pub struct Channel {
    pub language: String,
    pub copyright: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub title: String,
    pub link: String,
    pub description: String,
    #[serde(rename = "item")]
    pub items: Vec<Item>,
}

#[derive(Deserialize)]
pub struct Item {
    pub title: String,
    pub link: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub description: String,
    pub comments: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn make_rss_test() {
        let s = r#"
            <rss version="2.0">
                <channel>
                    <language>ja</language>
                    <copyright>sabinote</copyright>
                    <pubDate>2021-08-12T12:03:56.577Z</pubDate>
                    <title>タイトルです</title>
                    <link>リンクです</link>
                    <description>説明です</description>
                    <item>
                        <title>アイテムのタイトルです</title>
                        <link>アイテムへのリンクです</link>
                        <pubDate>2021-08-12T11:37:05.000Z</pubDate>
                        <description>アイテムの説明です</description>
                        <comments>アイテムへのコメントです</comments>
                    </item>
                </channel>
            </rss>
        "#;
        let rss = RSS::from_str(&s).unwrap();
        let channel = &rss.channel;
        assert_eq!(channel.language, "ja");
        assert_eq!(channel.copyright, "sabinote");
        assert_eq!(channel.pub_date, "2021-08-12T12:03:56.577Z");
        assert_eq!(channel.title, "タイトルです");
        assert_eq!(channel.link, "リンクです");
        assert_eq!(channel.description, "説明です");

        let items = &channel.items;
        assert_eq!(items.len(), 1);
        let item = items.get(0).unwrap();
        assert_eq!(item.title, "アイテムのタイトルです");
        assert_eq!(item.link, "アイテムへのリンクです");
        assert_eq!(item.pub_date, "2021-08-12T11:37:05.000Z");
        assert_eq!(item.description, "アイテムの説明です");
        assert_eq!(item.comments, "アイテムへのコメントです");
    }
}
