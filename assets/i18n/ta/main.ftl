# ── Title bar ──────────────────────────────────────────────────────────────────
title-app-name = NUS NextBus TUI
title-subtitle = NUS உள்நாட்டு பேருந்து சேவை

# ── Stop list panel ────────────────────────────────────────────────────────────
panel-bus-stops = பேருந்து நிறுத்தங்கள் ({ $count })
panel-favourites = ★ விருப்பமானவை ({ $count })

# ── Detail panel ───────────────────────────────────────────────────────────────
detail-title       = விவரங்கள்
detail-no-stops    = காட்ட எந்த நிறுத்தங்களும் இல்லை.
detail-loading     = ஏற்றப்படுகிறது...
detail-no-data     = இதுவரை தரவு இல்லை. புதுப்பிக்க [r] அழுத்தவும்.
detail-no-buses    = தற்போது எந்த பேருந்துகளும் சேவையில் இல்லை.
detail-refreshing  = புதுப்பிக்கப்படுகிறது...
detail-last-refreshed = கடைசியாக: { $elapsed }வி முன்பு   தானாக புதுப்பிக்கப்படும்: { $remaining }வி / { $total }வி
detail-last-fetched   = கடைசியாகப் பெறப்பட்டது: { $elapsed }வி முன்பு
detail-error          = ! { $message }

# ── Table column headers ────────────────────────────────────────────────────────
col-bus       = பேருந்து
col-next      = அடுத்து
col-following = பின்வரும்
col-plate     = எண்

# ── Arrival time values ─────────────────────────────────────────────────────────
arrival-arriving = வருகிறது
arrival-minutes  = { $minutes } நிமி

# ── Footer hints ────────────────────────────────────────────────────────────────
footer-normal        = [↑↓/j/k] நகர்த்த   [f] சேமி   [r] புதுப்பி   [/] தேடு   [s] அமைப்புகள்   [q] வெளியேறு
footer-jump          = தாவு: { $digits }_
footer-search        = வடிகட்ட தட்டச்சு செய்யவும்   [↑↓] தேர்வு   [↵] சரி   [Esc] ரத்து
footer-settings-nav  = [↑↓/j/k] தேர்வு   [↵/Space] மாற்று   [Esc/s] மூடு
footer-settings-edit = [0-9] உள்ளிடு   [⌫] நீக்கு   [↵] சரி   [Esc] ரத்து
footer-theme-picker  = [↑↓/j/k] தேர்வு   [↵] பயன்படுத்து   [Esc] மூடு

# ── Overlay titles ──────────────────────────────────────────────────────────────
search-title   = 🔍 தேடல்
theme-title    = 🎨 தீம்கள்
settings-title = ⚙ அமைப்புகள்

lang-picker-title = 🌐 மொழி
# ── Settings rows ───────────────────────────────────────────────────────────────
settings-interval-label   = தானியங்கி புதுப்பிப்பு இடைவெளி:
settings-interval-value   = [{ $seconds }வி]
settings-interval-editing = [{ $value }█]
settings-view-label       = இயல்புநிலை காட்சி:
settings-view-all         = [அனைத்து நிறுத்தங்கள்]
settings-view-favs        = [விருப்பமானவை]
settings-lang-label       = மொழி:
settings-lang-value       = [{ $name }]
settings-theme-mode-label = தீம் முறை:
settings-theme-mode-dark  = [இருண்ட]
settings-theme-mode-light = [வெளிர்]
settings-theme-mode-auto  = [தானியங்கு]
settings-hint-nav         = [↑↓/j/k] தேர்வு   [↵/Space] மாற்று   [Esc/s] மூடு
settings-hint-edit        = [0-9] உள்ளிடு   [⌫] நீக்கு   [↵] சரி   [Esc] ரத்து

# ── Status messages ─────────────────────────────────────────────────────────────
status-fav-added      = விருப்பமானவையில் சேர்க்கப்பட்டது ★
status-fav-removed    = விருப்பமானவையில் இருந்து நீக்கப்பட்டது
status-refreshing     = புதுப்பிக்கப்படுகிறது...
status-interval-set   = தானியங்கி புதுப்பிப்பு { $seconds }வி என அமைக்கப்பட்டது
status-view-set       = இயல்புநிலை காட்சி: { $view }
status-view-all       = அனைத்து நிறுத்தங்கள்
status-view-favs      = விருப்பமானவை
status-lang-set       = மொழி: { $name }
status-theme-mode-set = தீம் முறை: { $mode }
