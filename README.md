# 短影音字幕提取工具

給短影音部門使用的字幕提取工具，拖入影片即可產出字幕檔。

## 架構

```
影片 → ffmpeg 提取音訊 → whisper.cpp 語音辨識 → Gemini AI 校正/翻譯 → 輸出 SRT/ASS/TXT
```

## 技術棧

- **前端**: HTML/CSS/JS（無框架）
- **後端**: Rust + Tauri v2
- **語音辨識**: whisper.cpp（small 模型，CPU-only，零 Python 依賴）
- **AI 校正**: Gemini API（OpenAI 相容格式，支援 Proxy 和 Official 模式）
- **音訊處理**: ffmpeg

## 資料夾結構

```
├── src-tauri/              # Rust 後端
│   ├── src/
│   │   ├── lib.rs          # 程式入口
│   │   ├── commands.rs     # Tauri 命令（process_video, settings 等）
│   │   ├── audio.rs        # ffmpeg 音訊提取
│   │   ├── whisper.rs      # whisper.cpp 語音辨識
│   │   ├── gemini.rs       # Gemini API 呼叫
│   │   ├── subtitle.rs     # SRT/ASS/TXT 格式轉換
│   │   ├── settings.rs     # 設定檔管理
│   │   ├── bin_path.rs     # 二進位檔搜尋（ffmpeg, whisper-cpp）
│   │   └── progress.rs     # 進度事件發送
│   ├── capabilities/       # Tauri 權限設定
│   ├── icons/              # 應用程式圖示
│   ├── Cargo.toml
│   └── tauri.conf.json     # Tauri 設定檔
├── ui/                     # 前端介面
│   ├── index.html
│   ├── styles.css
│   ├── boot.js             # 載入畫面控制
│   ├── app.js              # 主程式邏輯
│   ├── modules/
│   │   ├── file.js         # 檔案選擇/拖放
│   │   ├── settings.js     # 設定 modal
│   │   ├── progress.js     # 進度顯示
│   │   └── log.js          # 日誌管理
│   └── fonts/              # 內嵌字型
└── README.md
```

## Windows 部署

### 前置需求（建置用電腦）

1. [Rust](https://rustup.rs/) — 安裝 rustup
2. [Node.js](https://nodejs.org/) 20+ — Tauri CLI 需要
3. Visual Studio Build Tools — C++ 編譯工具

### 需要準備的外部檔案

放在 `src-tauri/` 目錄下：

```
bin/
├── ffmpeg.exe          # 從 https://github.com/BtbN/FFmpeg-Builds/releases 下載 static 版
└── whisper-cpp.exe     # 在 Windows 上編譯 whisper.cpp (BUILD_SHARED_LIBS=OFF)
models/
└── ggml-small.bin      # 從 Hugging Face 下載 whisper small 模型
```

### 編譯 whisper.cpp（Windows）

```powershell
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp
cmake -B build -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF
cmake --build build --config Release
# 產出: build\bin\Release\whisper-cli.exe → 複製為 bin\whisper-cpp.exe
```

### 下載模型

```powershell
# 約 466MB
curl -L -o models/ggml-small.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
```

### 建置安裝檔

```powershell
npm install -g @tauri-apps/cli
cargo tauri build
# 產出: src-tauri/target/release/bundle/nsis/短影音字幕提取工具_1.0.0_x64-setup.exe
```

## 使用者操作

1. 安裝 `.exe`
2. 開啟 → 設定 → API 分頁 → 填入 Gemini API Key 和 Base URL
3. 拖入影片 → 選擇語言/格式 → 開始提取

## 設計原則

- 即開即用，不需要安裝 CUDA 或 Python
- 介面簡潔，面向非技術人員
- 支援中文、英文、日文、韓文語音辨識
- 輸出格式：SRT / ASS / TXT / 全部
