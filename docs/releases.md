# Phát hành

linear_issues:
  - https://linear.app/keios/issue/KEI-866/phat-hanh-github-releases-stable-beta-nightly
  - https://linear.app/keios/issue/KEI-867/cap-nhat-tu-djong-trong-app-stable-beta-nightly

Ba kênh GitHub Release. Tải installer tại [Releases](https://github.com/KeiosStarqua/komodoro/releases).

| Kênh | Khi nào | Tag | GitHub Release |
|------|---------|-----|----------------|
| **Stable** | Bản dùng hằng ngày | `vX.Y.Z` (không hậu tố) | Latest, không prerelease |
| **Beta** | Test trước khi stable | `vX.Y.Z-beta.N` | Prerelease |
| **Nightly** | Test `main` mới nhất | rolling `nightly` | Prerelease, bị ghi đè mỗi lần chạy |

Cập nhật **trong app** (Tauri updater) không nằm ở đây — issue [KEI-867](https://linear.app/keios/issue/KEI-867/cap-nhat-tu-djong-trong-app-stable-beta-nightly).

## Cắt Stable hoặc Beta

1. Đặt `version` trong `src-tauri/tauri.conf.json` (và workspace `Cargo.toml` nếu đổi semver).
2. Commit trên `main`.
3. Tag và push:

```sh
git tag v0.1.0
git push origin v0.1.0

git tag v0.1.0-beta.1
git push origin v0.1.0-beta.1
```

Workflow `.github/workflows/release.yml` build Linux (x64), macOS (Apple Silicon), Windows (x64) rồi upload lên Release.

Hoặc: Actions → **release** → Run workflow → chọn `stable` hoặc `beta` (beta gắn tag `v{version}-beta.{run_number}`).

Nếu workflow báo "Resource not accessible by integration": Settings → Actions → Workflow permissions → Read and write.

## Cắt Nightly

Chạy tự động:

- Mỗi ngày 02:00 UTC
- Mỗi push `main` đụng code app / workflow nightly
- Actions → **nightly** → Run workflow

Release `nightly` bị xóa rồi tạo lại. Không chiếm nhãn latest.

## Bundles

`cargo tauri build` (workflow gọi `cargo tauri`). Artifact điển hình:

- Linux: `.deb`, `.AppImage`
- macOS: `.dmg` (ad-hoc sign, chưa có chứng chỉ Apple)
- Windows: `.msi` / NSIS (chưa code-sign)

Cài local không qua GitHub:

```sh
cargo tauri build
```

Linux: cài gói hệ thống như README trước khi build.

## Updater (chưa bật)

Khi KEI-867 xong: thêm secret `TAURI_SIGNING_PRIVATE_KEY` (và password nếu có), pubkey trong `tauri.conf.json`, rồi bật `uploadUpdaterJson` trên action publish.
