# ── Title bar ──────────────────────────────────────────────────────────────────
title-app-name = NUS NextBus TUI
title-subtitle = Dịch vụ xe buýt nội bộ NUS

# ── Stop list panel ────────────────────────────────────────────────────────────
panel-bus-stops = Trạm xe buýt ({ $count })
panel-favourites = ★ Yêu thích ({ $count })

# ── Detail panel ───────────────────────────────────────────────────────────────
detail-title       = Chi tiết
detail-no-stops    = Không có trạm nào để hiển thị.
detail-loading     = Đang tải...
detail-no-data     = Chưa có dữ liệu. Nhấn [r] để làm mới.
detail-no-buses    = Hiện không có xe buýt nào đang hoạt động.
detail-refreshing  = Đang làm mới...
detail-last-refreshed = Lần cuối: { $elapsed } giây trước   Tự động làm mới trong: { $remaining } giây / { $total } giây
detail-last-fetched   = Lần tìm nạp cuối cùng: { $elapsed } giây trước
detail-error          = ! { $message }

# ── Table column headers ────────────────────────────────────────────────────────
col-bus       = Xe buýt
col-next      = Tiếp theo
col-following = Kế tiếp
col-plate     = Biển số

# ── Arrival time values ─────────────────────────────────────────────────────────
arrival-arriving = Đang đến
arrival-minutes  = { $minutes } phút

# ── Footer hints ────────────────────────────────────────────────────────────────
footer-normal        = [↑↓/j/k] Di chuyển   [f] Yêu thích   [r] Làm mới   [/] Tìm kiếm   [s] Cài đặt   [q] Thoát
footer-jump          = Nhảy: { $digits }_
footer-search        = Gõ để lọc   [↑↓] Điều hướng   [↵] Xác nhận   [Esc] Hủy
footer-settings-nav  = [↑↓/j/k] Điều hướng   [↵/Space] Chỉnh sửa/Chuyển đổi   [Esc/s] Đóng
footer-settings-edit = [0-9] Gõ   [⌫] Xóa   [↵] Xác nhận   [Esc] Hủy
footer-theme-picker  = [↑↓/j/k] Điều hướng   [↵] Áp dụng   [Esc] Đóng

# ── Overlay titles ──────────────────────────────────────────────────────────────
search-title   = 🔍 Tìm kiếm
theme-title    = 🎨 Chủ đề
settings-title = ⚙ Cài đặt

# ── Settings rows ───────────────────────────────────────────────────────────────
settings-interval-label   = Khoảng thời gian tự động làm mới:
settings-interval-value   = [{ $seconds } giây]
settings-interval-editing = [{ $value }█]
settings-view-label       = Chế độ xem mặc định:
settings-view-all         = [Tất cả các trạm]
settings-view-favs        = [Yêu thích]
settings-lang-label       = Ngôn ngữ:
settings-lang-value       = [{ $name }]
settings-theme-mode-label = Chế độ giao diện:
settings-theme-mode-dark  = [Tối]
settings-theme-mode-light = [Sáng]
settings-theme-mode-auto  = [Tự động]
settings-hint-nav         = [↑↓/j/k] Điều hướng   [↵/Space] Chỉnh sửa/Chuyển đổi   [Esc/s] Đóng
settings-hint-edit        = [0-9] Gõ   [⌫] Xóa   [↵] Xác nhận   [Esc] Hủy

# ── Status messages ─────────────────────────────────────────────────────────────
status-fav-added      = Đã thêm vào mục yêu thích ★
status-fav-removed    = Đã xóa khỏi mục yêu thích
status-refreshing     = Đang làm mới...
status-interval-set   = Tự động làm mới được đặt thành { $seconds } giây
status-view-set       = Chế độ xem mặc định được đặt thành: { $view }
status-view-all       = Tất cả các trạm
status-view-favs      = Yêu thích
status-lang-set       = Ngôn ngữ: { $name }
status-theme-mode-set = Chế độ giao diện: { $mode }
