# Icons

Згенеруйте іконки одною командою перед першою збіркою:

```powershell
cd desktop
npm run tauri icon path\to\source.png   # PNG 1024×1024, прозорий фон
```

Tauri створить:
- 32x32.png, 128x128.png, 128x128@2x.png
- icon.png, icon.ico (Windows), icon.icns (не використовується)
- Square*Logo.png для Windows Store

Без іконок збірка `tauri build` впаде з помилкою про відсутній `icon.ico`.