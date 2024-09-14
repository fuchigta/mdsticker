import { useEffect, useState } from "react";
import "./base.css";
import "./TrashBoxApp.css";
import { invoke } from "@tauri-apps/api";
import { VscReply, VscTrash } from "react-icons/vsc";
import { appWindow } from "@tauri-apps/api/window";

interface Sticker {
  checked: boolean;
  uuid: string;
  markdown: string;
  color: string;
  pinned: boolean;
}

const trashbox = {
  async loadStickers() {
    return await invoke<Sticker[]>("load_trashbox_stickers");
  },
  async deleteStickers(stickers: Sticker[]) {
    await invoke("delete_stickers", { stickers });
  },
  async recoverStickers(stickers: Sticker[]) {
    await invoke("recover_stickers", { stickers });
  },
};

function TrashBoxApp() {
  const [stickers, setStickers] = useState([] as Sticker[]);

  useEffect(() => {
    const load = () => trashbox.loadStickers().then((stickers: Sticker[]) => {
      setStickers(stickers.map((sticker) => ({ ...sticker, checked: false })));
    });

    load();

    appWindow.listen("reload", () => {
      load();
    })
  }, []);

  return (
    <div className="container">
      <header>
        <button
          onClick={(e) => {
            e.preventDefault();
            trashbox.deleteStickers(stickers.filter((s) => s.checked));
            setStickers(stickers.filter((s) => !s.checked));
          }}
        >
          <VscTrash />
        </button>
        <button
          onClick={(e) => {
            e.preventDefault();
            trashbox.recoverStickers(stickers.filter((s) => s.checked));
            setStickers(stickers.filter((s) => !s.checked));
          }}
        >
          <VscReply />
        </button>
      </header>
      <main>
        <table>
          <thead>
            <tr>
              <th>
                <input
                  type="checkbox"
                  checked={stickers.length !== 0 && stickers.every((s) => s.checked)}
                  onChange={(e) => {
                    setStickers(
                      stickers.map((s) => ({ ...s, checked: e.target.checked }))
                    );
                  }}
                />
              </th>
              <th>Markdown</th>
            </tr>
          </thead>
          <tbody>
            {stickers.map((sticker) => (
              <tr key={sticker.uuid}>
                <td>
                  <input
                    type="checkbox"
                    checked={sticker.checked}
                    onChange={(e) => {
                      setStickers(
                        stickers.map((s) => {
                          if (s.uuid == sticker.uuid) {
                            return { ...s, checked: !s.checked };
                          }

                          return s;
                        })
                      );
                    }}
                  />
                </td>
                <td>{sticker.markdown}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </main>
    </div>
  );
}

export default TrashBoxApp;
