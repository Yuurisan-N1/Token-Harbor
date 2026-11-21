<div align="center">

<img width="100%" alt="header" src="https://capsule-render.vercel.app/api?type=waving&height=210&text=Token%20Harbor%20Bot&fontAlign=50&fontAlignY=36&fontSize=56&desc=Auto%20Signup%20%7C%20Free%20Model%20Unlock%20%7C%20API%20Key%20%7C%20Chat%20and%20Usage%20Report&descAlign=50&descAlignY=58"/>

<img alt="typing" src="https://readme-typing-svg.demolab.com?font=Inter&size=18&duration=3000&pause=650&center=true&vCenter=true&width=900&lines=Auto+Signup+via+Disposable+Inbox+%7C+Turnstile+Solver;Email+Verification+%7C+Free+Model+Unlock;API+Key+Creation+%7C+Chat+%26+Usage+Report;Enroll+Mode+%7C+Multi-Key+%7C+Proxy+Support"/>

<p>
  <img alt="rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white"/>
  <img alt="platform" src="https://img.shields.io/badge/Platform-Token%20Harbor-111111"/>
  <img alt="build" src="https://github.com/Yuurisan-N1/Token-Harbor/actions/workflows/build.yml/badge.svg"/>
  <img alt="proxy" src="https://img.shields.io/badge/Proxy-Supported-111111"/>
  <img alt="author" src="https://img.shields.io/badge/by-Yuurisandesu-111111"/>
</p>

<p>
  <b>Token Harbor Bot</b> は Token Harbor 向けの Rust 製自動化クライアントやで。<br/>
  使い捨てメールボックスを開いて、Turnstile ソルバーで新しいアカウントを登録して、認証メールを確認して、無料モデルをアンロックして、API キーを作って、毎サイクルごとにチャットを1回送ってアカウントを維持しながら使用量ヘッダーも読み取るんや。複数アカウントを一気に登録するエンロールモードもあって、プロキシにも対応してるで。<br/>
  作って配布してるのは <b>Yuurisandesu</b> やで。
</p>

</div>

<hr/>

<p align="center">
  <img src="image/image.png" alt="プレビュー" width="100%"/>
</p>

<hr/>

---

## 目次

- [必要なもの](#必要なもの)
- [インストール](#インストール)
- [設定](#設定)
- [ボットの動かし方](#ボットの動かし方)
- [実行モード](#実行モード)
- [ビルド済みバイナリのダウンロード](#ビルド済みバイナリのダウンロード)
- [機能](#機能)
- [ファイル構成](#ファイル構成)
- [CI（継続的インテグレーション）](#ci継続的インテグレーション)
- [免責事項](#免責事項)

---

## 必要なもの

- Rust 安定版ツールチェーン（ソースからビルドするときだけ必要、[rustup.rs](https://rustup.rs) からインストールしてな）
- Git

---

## インストール

### Rust のインストール

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

確認：

```bash
rustc --version
```

**Windows:**

https://rustup.rs からインストーラーをダウンロードして実行したら、ターミナルを開き直して確認してな：

```cmd
rustc --version
```

**Termux (Android):**

```bash
pkg update && pkg install proot-distro
proot-distro install ubuntu
proot-distro login ubuntu
```

Ubuntu の中に入ったら：

```bash
apt update && apt install -y curl git build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

### クローンしてビルド

**リポジトリをクローン：**

```bash
git clone https://github.com/Yuurisan-N1/Token-Harbor.git
cd Token-Harbor
```

**リリースバイナリをビルド：**

```bash
./run.sh build
```

---

## 設定

### 1. API キー（data.txt）

`data.txt` には登録済みアカウントの API キーを1行ずつ書くんや。初回はファイルが空でもええで。ボットが自動でアカウント作って鍵を書き込んでくれるから。複数の鍵を先に用意したいならエンロールモードを使ってな。

### 2. プロキシ（proxy.txt）

`proxy.txt` にプロキシを1行ずつ書いてな（任意、空のままでも直接接続できるで）：

```
http://user:pass@host:port
socks5://user:pass@host:port
```

プロキシはアカウントにインデックス順で割り当てられるで。ログにはプロキシの認証情報はマスクして表示されるから安心してな。

### 3. ボットの設定（config.json）

`sleep_seconds` はサイクルとサイクルの間に何秒待つかを設定するんや。60〜86400秒の範囲に収まるように自動でクランプされるで。`timezone` を任意で設定すれば、登録時に使うタイムゾーンを変えられるで。デフォルトは3600秒（1時間）やで。

### 4. ソルバー（solver/solver.yaml）

`solver/solver.yaml` には登録時に使う Turnstile ソルバーのエンドポイントを書くんや。デフォルトの URL は最初から入ってるで：

```yaml
solver:
  url: https://solver.yuurisan.my.id
```

---

## ボットの動かし方

Linux / macOS / Termux では最初に実行権限を付けてな：

```bash
chmod +x run.sh
```

あとはモードを選んで実行するだけや：

```bash
./run.sh run            # 必要ならビルドして1サイクルをフォアグラウンドで動かす
./run.sh enroll <n>     # n 個のアカウントを登録して鍵を全部 data.txt に保存する
./run.sh bg             # 必要ならビルドしてバックグラウンドで動かす（ログは tokenharbor-bot.log に）
./run.sh bgprobe        # プローブをバックグラウンドで動かす
./run.sh screen         # 必要ならビルドして screen セッションで動かす
./run.sh stop           # バックグラウンドや screen セッションを止める
./run.sh probe          # サイト・ソルバー・各プロキシをチェックする
./run.sh selftest       # ネットワーク不要のオフラインチェック
./run.sh build          # ビルドだけして動かさへん
```

バックグラウンドモードのログはいつでも確認できるで：

```bash
tail -f tokenharbor-bot.log
```

screen モードはいつでもアタッチできるで：

```bash
screen -r tokenharbor-bot
```

フォアグラウンドで動かしてるときは `Ctrl+C` を1回押せばきれいに止まるで。

### 手動で動かすとき

```bash
# Linux / macOS / Termux
./target/release/tokenharbor-bot

# Windows
.\target\release\tokenharbor-bot.exe
```

---

## 実行モード

| モード | コマンド | 説明 |
|---|---|---|
| run | `./run.sh run` | 1サイクル実行：鍵がなければ登録して、チャットと使用量レポートを実行 |
| enroll | `./run.sh enroll <n>` | n 個のアカウントを登録して鍵を全部 data.txt に追記 |
| bg | `./run.sh bg` | run と同じだけどバックグラウンドでログファイルつき |
| bgprobe | `./run.sh bgprobe` | プローブをバックグラウンドで実行 |
| screen | `./run.sh screen` | run と同じだけど detach した screen セッションで実行 |
| probe | `./run.sh probe` | サイト・ソルバー・各プロキシをチェック |
| selftest | `./run.sh selftest` | ネットワーク不要のオフラインチェック |
| stop | `./run.sh stop` | バックグラウンドや screen セッションを止める |
| build | `./run.sh build` | ビルドだけ |

---

## ビルド済みバイナリのダウンロード

main へのプッシュのたびに GitHub Actions で3プラットフォーム向けに自動ビルドされて、リリースにアップされるで。

最新のバイナリはここからダウンロードしてな：
https://github.com/Yuurisan-N1/Token-Harbor/releases/latest

最新の成功したビルドを開いて、Artifacts のとこまでスクロールしてな。

| アーティファクト | プラットフォーム |
|---|---|
| `tokenharbor-bot-linux-x86_64` | Linux x86_64 |
| `tokenharbor-bot-windows-x86_64` | Windows x86_64 |
| `tokenharbor-bot-macos-aarch64` | macOS Apple Silicon |

**Linux / macOS のダウンロード後：**

```bash
chmod +x tokenharbor-bot-linux-x86_64
./tokenharbor-bot-linux-x86_64
```

**Windows：**

```
.\tokenharbor-bot-windows-x86_64.exe
```

動かす前に `data.txt`・`proxy.txt`・`config.json`・`solver/solver.yaml` と同じフォルダにバイナリを置いてな。

---

## 機能

### 使い捨てメールボックスで自動登録
`data.txt` に鍵がないときやエンロールモードのとき、ボットが自動で使い捨てメールボックスを開くんや。mail.tm → mail.io → mail.lol → Guerrilla Mail の順に試してくで。`th-` プレフィックスでランダムなメールアドレスを生成して、新しいパスワードとアカウントのタイムゾーンを入れて登録フォームを送信して、認証メールが届くまで待つんや。

### Turnstile ソルバー
登録リクエストのたびに `solver/solver.yaml` のエンドポイントで Cloudflare Turnstile トークンを自動で解くで。起動時とプローブモードでソルバーの接続確認もやってくれるで。

### メール認証
登録後にメールボックスをポーリングして、認証メールが届いたら確認リンクを踏んでくれるで。最初のポーリングがタイムアウトしたら、諦める前にリトライもするで。

### 無料モデルのアンロック
認証が終わったら、API キーがクレジットなしでモデルにアクセスできるよう無料モデルティアをアカウントに有効化するんや。

### API キーの作成
アカウントに名前つきの API キーを作って `data.txt` に追記するで。キーはマスクされた形でログに出るで。アカウントのメールアドレス・パスワード・鍵は `session.json` に保存されるで。

### チャットと使用量レポート
毎サイクル、ボットがモデルリストを読んで無料モデルを選んで、アカウントを維持するために短いチャットを1回送って、サーバーから使用量ヘッダーを読み取るんや。使用率とリセットまでのカウントダウンがアカウントごとにログに出るで。

### エンロールモード
`enroll <n>` で動かすと、ちょうど n 個のアカウントを1つずつ登録して、完了するたびに鍵を `data.txt` に書き込んでくで。各アカウントは `proxy.txt` のプロキシスロットを順番に使うで。

### プローブモード
プローブは登録ページ・モデルリストエンドポイント・ソルバーの接続・`proxy.txt` の全プロキシの出口国をチェックするで。登録したりファイルを変えたりはしいひんで。

### プロキシ対応
`proxy.txt` からプロキシを読み込んでアカウントに順番で割り当てるで。プロキシなしでも全然動くで。

### 自動カウントダウン
各サイクルが終わったら次のサイクルまでの時間を `HH:MM:SS` でリアルタイムに表示してくれるで。

---

## ファイル構成

```text
Token-Harbor/
├── .github/
│   └── workflows/
│       └── build.yml                    # CI: 3プラットフォームのマトリックスビルド
├── assets/
│   └── icon.ico                         # Windows バイナリのアイコン
├── solver/
│   └── solver.yaml                      # Turnstile ソルバーのエンドポイント URL
├── src/
│   ├── cli/
│   │   ├── dispatch.rs                  # CLI モードセレクターと使い方テキスト
│   │   ├── enroll.rs                    # エンロールモード：バッチアカウント登録
│   │   ├── mod.rs
│   │   ├── probe.rs                     # プローブモード：サイト・ソルバー・プロキシチェック
│   │   └── selftest.rs                  # オフラインのセルフテストチェック
│   ├── job/
│   │   ├── loop.rs                      # run・enroll モードのサイクルランナー
│   │   ├── mod.rs
│   │   └── onboard.rs                   # 登録フロー・API キー作成・チャットと使用量レポート
│   ├── net/
│   │   ├── client.rs                    # HTTP クライアントとクッキー処理
│   │   ├── egress.rs                    # プロキシパーサーと出口国チェック
│   │   ├── gate.rs                      # Turnstile ソルバーの WebSocket ゲート
│   │   └── mod.rs
│   ├── step/
│   │   ├── apikey.rs                    # API キー作成リクエスト
│   │   ├── freemode.rs                  # 無料モデルティア有効化リクエスト
│   │   ├── mailbox.rs                   # 使い捨てメールボックス：mail.tm, mail.io, mail.lol, Guerrilla
│   │   ├── mod.rs
│   │   ├── signup.rs                    # アカウント登録フォームと ID 生成
│   │   └── verify.rs                    # 認証リンクの踏み込みと確認
│   ├── support/
│   │   ├── banner.rs                    # ASCII バナーとターミナルタイトル
│   │   ├── cfg/
│   │   │   ├── loader.rs                # 設定・鍵リスト・プロキシのローダー
│   │   │   ├── mod.rs
│   │   │   └── schema.rs                # 設定スキーマとスリープ・タイムゾーン設定
│   │   ├── consts/
│   │   │   ├── app.rs                   # ファイル名定数とアカウント生成パラメーター
│   │   │   ├── mod.rs
│   │   │   └── net.rs                   # ベース URL・ソルバーパス・メールサービス URL
│   │   ├── error.rs                     # エラー型と Result エイリアス
│   │   ├── log/
│   │   │   ├── format.rs                # 色付きログヘルパーとカウントダウン
│   │   │   ├── init.rs                  # シグナルハンドラーのセットアップ
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── wire/
│   │   ├── endpoint.rs                  # API エンドポイントパスの定数
│   │   ├── form.rs                      # JSON リクエストボディのビルダー
│   │   ├── mod.rs
│   │   └── shape.rs                     # JSON レスポンスのパーサー
│   └── main.rs                          # エントリーポイント・Ctrl+C ハンドラー・ルートディレクトリリゾルバー
├── .gitignore
├── build.rs                             # ビルドスクリプト：Windows バイナリにアイコンを埋め込む
├── Cargo.lock
├── Cargo.toml                           # プロジェクトマニフェストと依存関係
├── config.json                          # スリープ時間と任意のタイムゾーン設定
├── data.txt                             # API キー（1行1つ、登録・エンロールで自動書き込み）
├── HOWTO.txt                            # クイックスタートガイド
├── LICENSE                              # GNU AGPL v3.0
├── Makefile                             # make ターゲット：build, release, run, start, clean, size
├── proxy.txt                            # プロキシリスト（任意）
├── README.md                            # このファイルや
├── run.sh                               # 実行ヘルパー：run, enroll, bg, bgprobe, screen, probe, selftest, stop, build
└── session.json                         # アカウントのメール・パスワード・鍵のキャッシュ（自動生成）
```

---

## CI（継続的インテグレーション）

GitHub Actions のワークフローは `main` または `master` へのプッシュ時と手動トリガーで動くで。3プラットフォームでリリースバイナリをビルドしてアーティファクトをアップロードするで。

| プラットフォーム | アーティファクト |
|---|---|
| Linux | `tokenharbor-bot-linux-x86_64` |
| Windows | `tokenharbor-bot-windows-x86_64` |
| macOS | `tokenharbor-bot-macos-aarch64` |

---

## 免責事項

このツールは学習目的と技術的な探求のために作ったもんやで。賢く使うてな、自己責任でよろしくやで。

---

<div align="center">
<img width="100%" alt="footer" src="https://capsule-render.vercel.app/api?type=waving&height=120&section=footer"/>
</div>