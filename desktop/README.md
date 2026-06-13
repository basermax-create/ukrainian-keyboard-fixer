# Raskladka Fix — Windows Tray App

Резидентна утиліта для Windows 10/11: виділяєте текст у будь-якому додатку,
тиснете глобальний хоткей (`Ctrl+Shift+U` за замовч.) — і «крякозябри»,
набрані англійською розкладкою, перетворюються на коректний український
текст прямо в полі вводу.

## Технології
- **Tauri 2.x** (Rust + WebView2)
- **React + Vite** — вікно налаштувань
- Плагіни: `global-shortcut`, `clipboard-manager`, `autostart`, `notification`
- `enigo` — емуляція Ctrl+C / Ctrl+V

## Структура
```
desktop/
├── src/                 # React UI (вікно налаштувань)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs      # tray, хоткей, оркестрація
│   │   └── layout.rs    # таблиця QWERTY ↔ ЙЦУКЕН
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html
├── package.json
└── vite.config.ts
```

## Збірка на Windows
Передумови (одноразово):
1. [Node.js 20+](https://nodejs.org)
2. [Rust toolchain](https://rustup.rs) (stable, MSVC target)
3. [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++)
4. WebView2 — вже встановлений у Windows 11; для Win10 ставиться з [evergreen-installer](https://developer.microsoft.com/microsoft-edge/webview2/)

Команди:
```powershell
cd desktop
npm install
npm run tauri dev      # розробка з гарячим перезавантаженням
npm run tauri build    # фінальна збірка → src-tauri/target/release/bundle/
```

Готовий `.msi` буде у `src-tauri/target/release/bundle/msi/`,
`.exe` (NSIS) — у `src-tauri/target/release/bundle/nsis/`.

## Як це працює
1. При запуску застосунок реєструє глобальний хоткей `Ctrl+Shift+U`.
2. По хоткею: зберігає поточний clipboard → емулює `Ctrl+C` → читає виділений текст.
3. Прогоняє текст через мапінг (`layout.rs`) у потрібному напрямку.
4. Записує результат у clipboard → емулює `Ctrl+V` → відновлює оригінальний clipboard.
5. Показує тихий toast «Конвертовано».

## Іконки
Перед першою збіркою згенеруйте іконки з PNG 1024×1024:
```powershell
npm run tauri icon path/to/icon.png
```
Це створить усі потрібні розміри в `src-tauri/icons/`.

## Налаштування
Tray menu → **Налаштування** відкриває вікно з опціями:
- Гаряча клавіша
- Напрямок (EN→UA / UA→EN / Авто)
- Автозапуск з Windows
- Показувати нотифікації

Налаштування зберігаються в `%APPDATA%/raskladka-fix/settings.json`.

## Відомі обмеження
- Поля паролів і деякі термінали блокують програмний `Ctrl+C` — програма покаже toast і скопіює конвертований текст у clipboard.
- Google Docs / Notion використовують власний clipboard layer — можливі затримки.
- Без підпису коду Windows SmartScreen показує попередження «Unknown publisher» при першому запуску — це нормально для dev-збірки.