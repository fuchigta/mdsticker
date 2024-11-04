import { useEffect, useState } from "react";
import "./base.css";
import "./TrashBoxApp.css";
import { invoke } from "@tauri-apps/api";
import { VscCheck, VscCheckAll, VscReply, VscTrash } from "react-icons/vsc";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import StickerMarkdown from "./StickerMarkdown";
const appWindow = getCurrentWebviewWindow()

interface Sticker {
  checked: boolean;
  uuid: string;
  markdown: string;
  color: string;
  pinned: boolean;
  updated_at: string;
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
    const load = () =>
      trashbox.loadStickers().then((stickers: Sticker[]) => {
        setStickers(
          stickers.map((sticker) => ({ ...sticker, checked: false }))
        );
      });

    load();

    appWindow.listen("reload", () => {
      load();
    });
  }, []);

  return (
    <div className="container">
      <header>
        <div className="controller">
          <button
              onClick={(e) => {
                e.preventDefault();

                if (stickers.every((s) => s.checked)) {
                  setStickers(stickers.map((s) => ({ ...s, checked: false })));
                  return;
                }

                setStickers(stickers.map((s) => ({ ...s, checked: true })));
              }}
            >
              <VscCheckAll />
          </button>
        </div>
        <div className="manager">
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
        </div>
      </header>
      <main>
        {stickers.map((sticker) => (
          <div key={sticker.uuid} className="trash-sticker" style={{backgroundColor: sticker.color}}>
            <label className="trash-sticker-header textarea">
              <div className="checkbox">
                <span className="check">{sticker.checked ? <VscCheck /> : " "}</span>
                <input
                  type="checkbox"
                  checked={sticker.checked}
                  onChange={() => {
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
              </div>
              <span>
                {sticker.updated_at}
              </span>
            </label>
            <div className="trash-sticker-body">
              <StickerMarkdown className="textarea" markdown={sticker.markdown} />
            </div>
          </div>
        ))}
      </main>
    </div>
  );
}

export default TrashBoxApp;
