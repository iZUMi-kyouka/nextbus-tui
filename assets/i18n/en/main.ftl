# ── Title bar ──────────────────────────────────────────────────────────────────
title-app-name = NUS NextBus TUI
title-subtitle = NUS Internal Shuttle Service

# ── Stop list panel ────────────────────────────────────────────────────────────
panel-bus-stops = Bus Stops ({ $count })
panel-favourites = ★ Favourites ({ $count })

# ── Detail panel ───────────────────────────────────────────────────────────────
detail-title       = Details
detail-no-stops    = No stops to display.
detail-loading     = Loading...
detail-no-data     = No data yet.  Press [r] to fetch.
detail-no-buses    = No buses currently in service.
detail-refreshing  = Refreshing...
detail-last-refreshed = Last: { $elapsed }s ago   Auto-refresh in: { $remaining }s / { $total }s
detail-last-fetched   = Last fetched: { $elapsed }s ago
detail-error          = ! { $message }

# ── Table column headers ────────────────────────────────────────────────────────
col-bus       = Bus
col-next      = Next
col-following = Following
col-plate     = Plate

# ── Arrival time values ─────────────────────────────────────────────────────────
arrival-arriving = Arriving
arrival-minutes  = { $minutes } min

# ── Footer hints ────────────────────────────────────────────────────────────────
footer-normal        = [↑↓/j/k] Move   [f] Favourite   [r] Refresh   [/] Search   [s] Settings   [q] Quit
footer-jump          = Jump: { $digits }_
footer-search        = Type to filter   [↑↓] Navigate   [↵] Confirm   [Esc] Cancel
footer-settings-nav  = [↑↓/j/k] Navigate   [↵/Space] Edit/Toggle   [Esc/s] Close
footer-settings-edit = [0-9] Type   [⌫] Delete   [↵] Confirm   [Esc] Cancel
footer-theme-picker  = [↑↓/j/k] Navigate   [↵] Apply   [Esc] Close

# ── Overlay titles ──────────────────────────────────────────────────────────────
search-title   = 🔍 Search
theme-title    = 🎨 Themes
settings-title = ⚙ Settings

# ── Settings rows ───────────────────────────────────────────────────────────────
settings-interval-label   = Auto-refresh interval:
settings-interval-value   = [{ $seconds }s]
settings-interval-editing = [{ $value }█]
settings-view-label       = Default view:
settings-view-all         = [All stops]
settings-view-favs        = [Favourites]
settings-lang-label       = Language:
settings-lang-value       = [{ $name }]
settings-hint-nav         = [↑↓/j/k] Navigate   [↵/Space] Edit/Toggle   [Esc/s] Close
settings-hint-edit        = [0-9] Type   [⌫] Delete   [↵] Confirm   [Esc] Cancel

# ── Status messages ─────────────────────────────────────────────────────────────
status-fav-added      = Added to favourites ★
status-fav-removed    = Removed from favourites
status-refreshing     = Refreshing...
status-interval-set   = Auto-refresh set to { $seconds }s
status-view-set       = Default view set to: { $view }
status-view-all       = All stops
status-view-favs      = Favourites
status-lang-set       = Language: { $name }
