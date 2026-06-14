import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Direction = "en_to_ua" | "ua_to_en" | "auto";

type Settings = {
  hotkey: string;
  direction: Direction;
  autostart: boolean;
  notifications: boolean;
  first_run: boolean;
};

const DEFAULTS: Settings = {
  hotkey: "CommandOrControl+Shift+U",
  direction: "auto",
  autostart: false,
  notifications: true,
  first_run: true,
};

export function App() {
  const [s, setS] = useState<Settings>(DEFAULTS);
  const [loaded, setLoaded] = useState(false);
  const [step, setStep] = useState(0);

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

  // ---------- First-run wizard ----------
  if (s.first_run) {
    const steps = [
      {
        title: "Вітаємо у Raskladka Fix!",
        body: (
          <p className="hint" style={{ fontSize: 14, lineHeight: 1.5 }}>
            Виділіть текст у будь-якому додатку, натисніть гарячу клавішу — і «крякозябри»,
            набрані не тією розкладкою, перетворяться на коректний текст.
          </p>
        ),
      },
      {
        title: "Оберіть гарячу клавішу",
        body: (
          <div className="row">
            <div>
              <div className="label">Hotkey</div>
              <div className="hint">За замовч. <span className="kbd">Ctrl+Shift+U</span></div>
            </div>
            <input
              type="text"
              value={s.hotkey}
              onChange={(e) => update({ hotkey: e.target.value })}
              style={{ width: 220 }}
            />
          </div>
        ),
      },
      {
        title: "Напрямок конвертації",
        body: (
          <div className="row">
            <div>
              <div className="label">Як визначати?</div>
              <div className="hint">«Авто» — за вмістом виділення</div>
            </div>
            <select
              value={s.direction}
              onChange={(e) => update({ direction: e.target.value as Direction })}
            >
              <option value="auto">Авто</option>
              <option value="en_to_ua">EN → українська</option>
              <option value="ua_to_en">UA → англійська</option>
            </select>
          </div>
        ),
      },
      {
        title: "Поведінка програми",
        body: (
          <>
            <div className="row">
              <div>
                <div className="label">Автозапуск з Windows</div>
                <div className="hint">Запускати в треї при вході</div>
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
                <div className="label">Нотифікації</div>
                <div className="hint">Toast після конвертації</div>
              </div>
              <div
                className={`toggle ${s.notifications ? "on" : ""}`}
                onClick={() => update({ notifications: !s.notifications })}
                role="switch"
                aria-checked={s.notifications}
              />
            </div>
          </>
        ),
      },
    ];
    const current = steps[step];
    const isLast = step === steps.length - 1;
    return (
      <main>
        <div className="hint" style={{ fontSize: 11, marginBottom: 4 }}>
          Крок {step + 1} з {steps.length}
        </div>
        <h1 style={{ marginTop: 0 }}>{current.title}</h1>
        <div style={{ minHeight: 180 }}>{current.body}</div>
        <div style={{ display: "flex", justifyContent: "space-between", marginTop: 24, gap: 8 }}>
          <button
            onClick={() => setStep((n) => Math.max(0, n - 1))}
            disabled={step === 0}
            style={{ padding: "8px 16px" }}
          >
            Назад
          </button>
          <button
            onClick={async () => {
              if (isLast) {
                await update({ first_run: false });
              } else {
                setStep((n) => n + 1);
              }
            }}
            style={{ padding: "8px 16px", fontWeight: 600 }}
          >
            {isLast ? "Готово" : "Далі"}
          </button>
        </div>
      </main>
    );
  }

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