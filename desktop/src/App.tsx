import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Direction = "en_to_ua" | "ua_to_en" | "auto";

type Settings = {
  hotkey: string;
  direction: Direction;
  autostart: boolean;
  notifications: boolean;
};

const DEFAULTS: Settings = {
  hotkey: "CommandOrControl+Shift+U",
  direction: "auto",
  autostart: false,
  notifications: true,
};

export function App() {
  const [s, setS] = useState<Settings>(DEFAULTS);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((v) => setS({ ...DEFAULTS, ...v }))
      .catch(() => {})
      .finally(() => setLoaded(true));
  }, []);

  const update = async (patch: Partial<Settings>) => {
    const next = { ...s, ...patch };
    setS(next);
    await invoke("save_settings", { settings: next });
  };

  if (!loaded) return null;

  return (
    <main>
      <h1>Налаштування Raskladka Fix</h1>

      <div className="row">
        <div>
          <div className="label">Гаряча клавіша</div>
          <div className="hint">За замовч. <span className="kbd">Ctrl+Shift+U</span></div>
        </div>
        <input
          type="text"
          value={s.hotkey}
          onChange={(e) => update({ hotkey: e.target.value })}
          style={{ width: 220 }}
        />
      </div>

      <div className="row">
        <div>
          <div className="label">Напрямок конвертації</div>
          <div className="hint">«Авто» визначає напрямок за вмістом</div>
        </div>
        <select
          value={s.direction}
          onChange={(e) => update({ direction: e.target.value as Direction })}
        >
          <option value="auto">Авто</option>
          <option value="en_to_ua">EN розкладка → українська</option>
          <option value="ua_to_en">UA розкладка → англійська</option>
        </select>
      </div>

      <div className="row">
        <div>
          <div className="label">Автозапуск з Windows</div>
          <div className="hint">Запускати в треї при вході в систему</div>
        </div>
        <div
          className={`toggle ${s.autostart ? "on" : ""}`}
          onClick={() => update({ autostart: !s.autostart })}
          role="switch"
          aria-checked={s.autostart}
        />
      </div>

      <div className="row">
        <div>
          <div className="label">Показувати нотифікації</div>
          <div className="hint">Toast після кожної конвертації</div>
        </div>
        <div
          className={`toggle ${s.notifications ? "on" : ""}`}
          onClick={() => update({ notifications: !s.notifications })}
          role="switch"
          aria-checked={s.notifications}
        />
      </div>

      <div style={{ marginTop: 24, fontSize: 12, color: "#8a8a90" }}>
        v0.1.0 — закрийте вікно, програма продовжить працювати в треї.
      </div>
    </main>
  );
}