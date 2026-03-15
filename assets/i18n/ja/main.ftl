# ── タイトルバー ────────────────────────────────────────────────────────────────
title-app-name = NUS NextBus TUI
title-subtitle = NUS 内部シャトルサービス

# ── バス停リスト ────────────────────────────────────────────────────────────────
panel-bus-stops = バス停 ({ $count })
panel-favourites = ★ お気に入り ({ $count })

# ── 詳細パネル ─────────────────────────────────────────────────────────────────
detail-title       = 詳細
detail-no-stops    = 表示するバス停がありません。
detail-loading     = 読み込み中...
detail-no-data     = データなし。[r] を押して取得してください。
detail-no-buses    = 現在運行中のバスはありません。
detail-refreshing  = 更新中...
detail-last-refreshed = 最終: { $elapsed }秒前   次の更新: { $remaining }秒 / { $total }秒
detail-last-fetched   = 最終取得: { $elapsed }秒前
detail-error          = ! { $message }

# ── テーブル列ヘッダー ──────────────────────────────────────────────────────────
col-bus       = バス
col-next      = 次
col-following = 次々
col-plate     = ナンバー

# ── 到着時刻 ──────────────────────────────────────────────────────────────────
arrival-arriving = まもなく
arrival-minutes  = { $minutes }分

# ── フッターヒント ─────────────────────────────────────────────────────────────
footer-normal        = [↑↓/j/k] 移動   [f] お気に入り   [r] 更新   [/] 検索   [s] 設定   [q] 終了
footer-jump          = 移動: { $digits }_
footer-search        = 入力してフィルター   [↑↓] 移動   [↵] 確定   [Esc] キャンセル
footer-settings-nav  = [↑↓/j/k] 移動   [↵/Space] 編集/切替   [Esc/s] 閉じる
footer-settings-edit = [0-9] 入力   [⌫] 削除   [↵] 確定   [Esc] キャンセル
footer-theme-picker  = [↑↓/j/k] 移動   [↵] 適用   [Esc] 閉じる

# ── オーバーレイタイトル ────────────────────────────────────────────────────────
search-title   = 🔍 検索
theme-title    = 🎨 テーマ
settings-title = ⚙ 設定

# ── 設定行 ────────────────────────────────────────────────────────────────────
settings-interval-label   = 自動更新間隔:
settings-interval-value   = [{ $seconds }秒]
settings-interval-editing = [{ $value }█]
settings-view-label       = デフォルト表示:
settings-view-all         = [全バス停]
settings-view-favs        = [お気に入り]
settings-lang-label       = 言語:
settings-lang-value       = [{ $name }]
settings-font-label       = フォント:
settings-font-value       = [{ $font }]
settings-hint-nav         = [↑↓/j/k] 移動   [↵/Space] 編集/切替   [Esc/s] 閉じる
settings-hint-edit        = [0-9] 入力   [⌫] 削除   [↵] 確定   [Esc] キャンセル

# ── ステータスメッセージ ────────────────────────────────────────────────────────
status-fav-added    = お気に入りに追加しました ★
status-fav-removed  = お気に入りから削除しました
status-refreshing   = 更新中...
status-interval-set = 自動更新を { $seconds }秒に設定しました
status-view-set     = デフォルト表示を変更しました: { $view }
status-view-all     = 全バス停
status-view-favs    = お気に入り
status-lang-set     = 言語: { $name } — フォント: { $font }
