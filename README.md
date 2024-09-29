# gakusai2024-backend

## 実装方法

[ガイド](./docs/guide.md)を参考にしてください。

## commit messageのルール

以下の記事を参考にcommit messageのタイトルを書いてください

<https://qiita.com/konatsu_p/items/dfe199ebe3a7d2010b3e>

## PRの出し方

GitHubフローを採用します。

masterブランチから`feature/issue番号-ブランチ名`という形式でissueごとにブランチを切り、PRを出します。

CIが通り1人以上にapproveされたらmergeされます。

レビュー前に次のタスクに取り掛かりたい場合、前のタスクに依存するタスクの場合はそこからブランチを切ってください。
