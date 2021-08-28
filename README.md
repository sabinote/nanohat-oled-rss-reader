# nanohat-oled-rss-reader
 
## 概要
 
[NEO Metal Complete Kit](https://www.friendlyarm.com/index.php?route=product/product&path=85&product_id=190)で動作するYahoo!JapanのRSSリーダーです。  
キット付属NanoHatOLED搭載のボタンで操作でき、取得したRSSの一覧をカテゴリごとOLCDに表示します。

| カテゴリ一覧 | タイトル一覧 | 本文 |
|----|----|----|
| <img src="https://user-images.githubusercontent.com/71957989/131208707-a41944aa-ecc8-4b92-b550-1b6bea07375f.JPG" width="320"> | <img src="https://user-images.githubusercontent.com/71957989/131208704-d5238cdc-f836-42d6-a206-3afede8bfa90.JPG" width="320"> | <img src="https://user-images.githubusercontent.com/71957989/131208706-6fd281ef-c3a8-4b6b-ac07-beec6081f4a5.JPG" width="320"> |

## 機能
 
- ボタン操作
- OLED表示
 
## 必要要件
 
- [NEO Metal Complete Kit](https://www.friendlyarm.com/index.php?route=product/product&path=85&product_id=190)で動作確認済み
- Rustがインストールされている環境
 
## 起動
 
``` 
$ git clone https://github.com/sabinote/nanohat-oled-rss-reader.git
$ cd nanohat-oled-rss-reader
$ cargo run --release
```

## 操作方法

| F1ボタン | F2ボタン | F3ボタン |
|----|----|----|
| ↓ | 決定 | ↑ |
 
