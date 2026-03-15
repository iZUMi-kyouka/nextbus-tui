# ── Title bar ──────────────────────────────────────────────────────────────────
title-app-name = NUS NextBus TUI
title-subtitle = Perkhidmatan Bas Dalaman NUS

# ── Stop list panel ────────────────────────────────────────────────────────────
panel-bus-stops = Hentian Bas ({ $count })
panel-favourites = ★ Kegemaran ({ $count })

# ── Detail panel ───────────────────────────────────────────────────────────────
detail-title       = Maklumat
detail-no-stops    = Tiada hentian untuk dipaparkan.
detail-loading     = Sedang memuat...
detail-no-data     = Tiada data lagi. Tekan [r] untuk segar semula.
detail-no-buses    = Tiada bas yang sedang beroperasi.
detail-refreshing  = Sedang menyegar semula...
detail-last-refreshed = Terakhir: { $elapsed }s lepas   Segar semula automatik dalam: { $remaining }s / { $total }s
detail-last-fetched   = Terakhir diambil: { $elapsed }s lepas
detail-error          = ! { $message }

# ── Table column headers ────────────────────────────────────────────────────────
col-bus       = Bas
col-next      = Seterusnya
col-following = Berikutnya
col-plate     = Plat

# ── Arrival time values ─────────────────────────────────────────────────────────
arrival-arriving = Tiba
arrival-minutes  = { $minutes } min

# ── Footer hints ────────────────────────────────────────────────────────────────
footer-normal        = [↑↓/j/k] Gerak   [f] Kegemaran   [r] Segar semula   [/] Cari   [s] Tetapan   [q] Keluar
footer-jump          = Lompat: { $digits }_
footer-search        = Taip untuk tapis   [↑↓] Navigasi   [↵] Sahkan   [Esc] Batal
footer-settings-nav  = [↑↓/j/k] Navigasi   [↵/Space] Edit/Togol   [Esc/s] Tutup
footer-settings-edit = [0-9] Taip   [⌫] Padam   [↵] Sahkan   [Esc] Batal
footer-theme-picker  = [↑↓/j/k] Navigasi   [↵] Guna   [Esc] Tutup

# ── Overlay titles ──────────────────────────────────────────────────────────────
search-title   = 🔍 Cari
theme-title    = 🎨 Tema
settings-title = ⚙ Tetapan

# ── Settings rows ───────────────────────────────────────────────────────────────
settings-interval-label   = Sela segar semula automatik:
settings-interval-value   = [{ $seconds }s]
settings-interval-editing = [{ $value }█]
settings-view-label       = Paparan lalai:
settings-view-all         = [Semua hentian]
settings-view-favs        = [Kegemaran]
settings-lang-label       = Bahasa:
settings-lang-value       = [{ $name }]
settings-hint-nav         = [↑↓/j/k] Navigasi   [↵/Space] Edit/Togol   [Esc/s] Tutup
settings-hint-edit        = [0-9] Taip   [⌫] Padam   [↵] Sahkan   [Esc] Batal

# ── Status messages ─────────────────────────────────────────────────────────────
status-fav-added      = Ditambah ke kegemaran ★
status-fav-removed    = Dibuang daripada kegemaran
status-refreshing     = Sedang menyegar semula...
status-interval-set   = Segar semula automatik ditetapkan kepada { $seconds }s
status-view-set       = Paparan lalai ditetapkan kepada: { $view }
status-view-all       = Semua hentian
status-view-favs      = Kegemaran
status-lang-set       = Bahasa: { $name }
