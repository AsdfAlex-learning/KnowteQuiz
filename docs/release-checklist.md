# KnowteQuiz Release Checklist

Use this list before tagging and publishing a release.

## Pre-release Checks

- [ ] All CI checks pass (frontend type-check, unit tests, Rust tests, clippy, smoke test)
- [ ] `npm run build` succeeds with no errors
- [ ] `cargo build --release` succeeds in `src-tauri/`
- [ ] `npm run format:check` passes (no unformatted files)
- [ ] Version bump in `src-tauri/Cargo.toml` and `package.json`
- [ ] Update `CHANGELOG.md` (if maintained)

## Manual Smoke Tests (Windows)

- [ ] **Launch**: App starts and renders the three-column layout
- [ ] **Folder**: Browse and select a notes folder — file tree populates
- [ ] **Reading**: Click a markdown file — content renders with syntax highlighting and KaTeX
- [ ] **Search**: Ctrl+F in reader — search bar appears and finds text
- [ ] **TOC**: Click ☰ button — outline sidebar shows headings
- [ ] **LLM Connection**: Settings → Test Connection → shows connected
- [ ] **Probe**: Settings → Probe LLM → shows model info and capabilities
- [ ] **Quiz Basic**: Select a note → Start Quiz → questions appear and can be answered
- [ ] **Quiz Advanced**: Start quiz in advanced mode → submit reasoning → diagnosis flows
- [ ] **Mistakes**: Answer wrong → mistake appears in Error Book
- [ ] **Mistake Search**: Error Book → search by question text
- [ ] **Mistake Filter**: Filter by mode (Basic/Advanced), by note, by blind-spot tag
- [ ] **Mistake Review**: Mark a mistake as reviewed → review count increments
- [ ] **Mistake Export**: Error Book → JSON/MD export buttons work
- [ ] **Backup**: Settings → Backup Data Now → shows success
- [ ] **Restore**: Settings → Restore Latest Backup → confirm dialog → shows success
- [ ] **Open Folder**: Settings → Open Folder → file explorer opens
- [ ] **Session Cleanup**: Settings → Clean Up Old Sessions → shows result
- [ ] **Layout Persist**: Resize columns, close and reopen → widths restored
- [ ] **Settings Persist**: Change a setting, restart app → setting retained

## Web Mode (Optional)

- [ ] `node start.cjs --web` starts and serves on localhost:14200
- [ ] All API endpoints respond (settings, notes, mistakes, status)
- [ ] Quiz generation via web mode works (requires LLM)
- [ ] Settings write → restart → verify persistence

## Post-release

- [ ] Create GitHub release with changelog
- [ ] Tag the release (`git tag vX.Y.Z`)
- [ ] Verify release assets are attached (if using Tauri bundler)
