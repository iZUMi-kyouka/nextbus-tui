# ── Title bar ──────────────────────────────────────────────────────────────────
title-app-name = NUS NextBus TUI
title-subtitle = NUS 內部接駁車服務

# ── Stop list panel ────────────────────────────────────────────────────────────
panel-bus-stops = 公車站 ({ $count })
panel-favourites = ★ 最愛 ({ $count })

# ── Detail panel ───────────────────────────────────────────────────────────────
detail-title       = 詳細資訊
detail-no-stops    = 無站點可顯示。
detail-loading     = 載入中...
detail-no-data     = 尚無資料。按 [r] 重新整理。
detail-no-buses    = 目前無公車服務。
detail-refreshing  = 重新整理中...
detail-last-refreshed = 上次: { $elapsed }秒前   自動重新整理於: { $remaining }秒 / { $total }秒
detail-last-fetched   = 上次擷取: { $elapsed }秒前
detail-error          = ! { $message }

# ── Table column headers ────────────────────────────────────────────────────────
col-bus       = 公車
col-next      = 下一班
col-following = 再下一班
col-plate     = 車牌

# ── Arrival time values ─────────────────────────────────────────────────────────
arrival-arriving = 即將抵達
arrival-minutes  = { $minutes } 分鐘

# ── Footer hints ────────────────────────────────────────────────────────────────
footer-normal        = [↑↓/j/k] 移動   [f] 加入最愛   [r] 重新整理   [/] 搜尋   [s] 設定   [q] 離開
footer-jump          = 跳至: { $digits }_
footer-search        = 輸入以篩選   [↑↓] 導覽   [↵] 確認   [Esc] 取消
footer-settings-nav  = [↑↓/j/k] 導覽   [↵/Space] 編輯/切換   [Esc/s] 關閉
footer-settings-edit = [0-9] 輸入   [⌫] 刪除   [↵] 確認   [Esc] 取消
footer-theme-picker  = [↑↓/j/k] 導覽   [↵] 套用   [Esc] 關閉

# ── Overlay titles ──────────────────────────────────────────────────────────────
search-title   = 🔍 搜尋
theme-title    = 🎨 主題
settings-title = ⚙ 設定

lang-picker-title = 🌐 語言
# ── Settings rows ───────────────────────────────────────────────────────────────
settings-interval-label   = 自動重新整理間隔:
settings-interval-value   = [{ $seconds }秒]
settings-interval-editing = [{ $value }█]
settings-view-label       = 預設檢視:
settings-view-all         = [所有站點]
settings-view-favs        = [最愛]
settings-lang-label       = 語言:
settings-lang-value       = [{ $name }]
settings-theme-mode-label = 主題模式:
settings-theme-mode-dark  = [深色]
settings-theme-mode-light = [淺色]
settings-theme-mode-auto  = [自動]
settings-hint-nav         = [↑↓/j/k] 導覽   [↵/Space] 編輯/切換   [Esc/s] 關閉
settings-hint-edit        = [0-9] 輸入   [⌫] 刪除   [↵] 確認   [Esc] 取消

# ── Status messages ─────────────────────────────────────────────────────────────
status-fav-added      = 已新增至最愛 ★
status-fav-removed    = 已從最愛移除
status-refreshing     = 重新整理中...
status-interval-set   = 自動重新整理設定為 { $seconds }秒
status-view-set       = 預設檢視設定為: { $view }
status-view-all       = 所有站點
status-view-favs      = 最愛
status-lang-set       = 語言: { $name }
status-theme-mode-set = 主題模式: { $mode }

# ── SG 公共巴士模式 ──────────────────────────────────────────────────────────────
title-mode-sg            = SG 公共巴士
title-switch-hint-sg     = [Tab] SG 公共巴士
title-switch-hint-nus    = [Tab] NUS 校園

sg-panel-stops-title    = SG 巴士站
sg-panel-stops          = 巴士站 ({ $count })
sg-panel-favs           = ★ 收藏 ({ $count })
sg-stops-loading        = 正在載入巴士站... (已載入 { $count } 個)
sg-stops-error          = 載入失敗: { $message }

sg-detail-no-service    = 此站暫無巴士服務。
sg-detail-loading       = 正在取得到站資訊...
sg-detail-no-data       = 尚無資料。按 [r] 重新整理。
sg-detail-error         = ! { $message }

sg-col-bus   = 巴士
sg-col-opr   = 營運
sg-col-next  = 下一班
sg-col-2nd   = 後續
sg-col-load  = 載客
sg-col-type  = 類型

settings-mode-label  = 預設模式:
settings-mode-nus    = [NUS 校園]
settings-mode-sg     = [SG 公共巴士]

status-mode-sg  = 已切換至 SG 公共巴士
status-mode-nus = 已切換至 NUS 校園

footer-normal-sg = [↑↓/j/k] 移動   [f] 收藏   [r] 重整   [/] 搜尋   [Tab] NUS   [s] 設定   [q] 退出
