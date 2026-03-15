# ── Title bar ──────────────────────────────────────────────────────────────────
title-app-name = NUS NextBus TUI
title-subtitle = NUS 内部班车服务

# ── Stop list panel ────────────────────────────────────────────────────────────
panel-bus-stops = 巴士站 ({ $count })
panel-favourites = ★ 收藏 ({ $count })

# ── Detail panel ───────────────────────────────────────────────────────────────
detail-title       = 详情
detail-no-stops    = 无站点显示。
detail-loading     = 加载中...
detail-no-data     = 暂无数据。按 [r] 刷新。
detail-no-buses    = 当前无巴士运行。
detail-refreshing  = 刷新中...
detail-last-refreshed = 上次: { $elapsed }秒前   自动刷新: { $remaining }秒 / { $total }秒
detail-last-fetched   = 上次获取: { $elapsed }秒前
detail-error          = ! { $message }

# ── Table column headers ────────────────────────────────────────────────────────
col-bus       = 巴士
col-next      = 下一班
col-following = 后续
col-plate     = 车牌

# ── Arrival time values ─────────────────────────────────────────────────────────
arrival-arriving = 即将到达
arrival-minutes  = { $minutes } 分

# ── Footer hints ────────────────────────────────────────────────────────────────
footer-normal        = [↑↓/j/k] 移动   [f] 收藏   [r] 刷新   [/] 搜索   [s] 设置   [q] 退出
footer-jump          = 跳转: { $digits }_
footer-search        = 输入筛选   [↑↓] 导航   [↵] 确认   [Esc] 取消
footer-settings-nav  = [↑↓/j/k] 导航   [↵/Space] 编辑/切换   [Esc/s] 关闭
footer-settings-edit = [0-9] 输入   [⌫] 删除   [↵] 确认   [Esc] 取消
footer-theme-picker  = [↑↓/j/k] 导航   [↵] 应用   [Esc] 关闭

# ── Overlay titles ──────────────────────────────────────────────────────────────
search-title   = 🔍 搜索
theme-title    = 🎨 主题
settings-title = ⚙ 设置

# ── Settings rows ───────────────────────────────────────────────────────────────
settings-interval-label   = 自动刷新间隔:
settings-interval-value   = [{ $seconds }秒]
settings-interval-editing = [{ $value }█]
settings-view-label       = 默认视图:
settings-view-all         = [所有站点]
settings-view-favs        = [收藏夹]
settings-lang-label       = 语言:
settings-lang-value       = [{ $name }]
settings-hint-nav         = [↑↓/j/k] 导航   [↵/Space] 编辑/切换   [Esc/s] 关闭
settings-hint-edit        = [0-9] 输入   [⌫] 删除   [↵] 确认   [Esc] 取消

# ── Status messages ─────────────────────────────────────────────────────────────
status-fav-added      = 已添加到收藏夹 ★
status-fav-removed    = 已从收藏夹移除
status-refreshing     = 刷新中...
status-interval-set   = 自动刷新设为 { $seconds }秒
status-view-set       = 默认视图设为: { $view }
status-view-all       = 所有站点
status-view-favs      = 收藏夹
status-lang-set       = 语言: { $name }
