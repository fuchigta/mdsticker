import { Fragment, useEffect, useRef, useState } from "react";
import "./base.css";
import "./App.css";
import { invoke } from "@tauri-apps/api";
import {
  VscEdit,
  VscNewFile,
  VscPin,
  VscPinned,
  VscSave,
  VscSymbolColor,
  VscTrash,
} from "react-icons/vsc";
import Picker from "@emoji-mart/react";
import data from "@emoji-mart/data";
import StickerMarkdown from "./StickerMarkdown";
import { MdEmojiEmotions } from "react-icons/md";

interface Sticker {
  uuid: string;
  markdown: string;
  color: string;
  pinned: boolean;
}

const sticker = {
  async create() {
    await invoke("new_sticker");
  },
  async load() {
    return await invoke<Sticker>("load_sticker");
  },
  async saveMarkdown(markdown: string) {
    await invoke("save_sticker_markdown", { markdown });
  },
  async saveColor(color: string) {
    await invoke("save_sticker_color", { color });
  },
  async remove() {
    await invoke("remove_sticker");
  },
  async togglePinned() {
    await invoke("toggle_sticker_pinned");
  },
};

function App() {
  const [editting, setEditting] = useState(false);
  const [markdown, setMarkdown] = useState("");
  const [color, setColor] = useState("");
  const [pinned, setPinned] = useState(false);
  const [isPickerOpen, setPickerOpen] = useState(false);
  const textArea = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    sticker.load().then(({ markdown, color, pinned }: Sticker) => {
      setMarkdown(markdown);
      setColor(color);
      setPinned(pinned);
    });
  }, []);

  return (
    <div className="container" style={{ backgroundColor: color }}>
      <header>
        <div className="controller">
          <button
            onClick={(e) => {
              e.preventDefault();
              setEditting(!editting);
              sticker.saveMarkdown(markdown);
            }}
          >
            {editting ? <VscSave /> : <VscEdit />}
          </button>
          <button
            onClick={(e) => {
              e.preventDefault();
              setPinned(!pinned);
              sticker.togglePinned();
            }}
          >
            {pinned ? <VscPinned /> : <VscPin />}
          </button>
          <div className="color-button">
            <button>
              <VscSymbolColor />
            </button>
            <input
              type="color"
              onChange={(e) => {
                e.preventDefault();
                setColor(e.target.value);
                sticker.saveColor(e.target.value);
              }}
            />
          </div>
          <button
            className="emoji-picker-button"
            onClick={(e) => {
              e.preventDefault();
              setPickerOpen(!isPickerOpen);
            }}
          >
            <MdEmojiEmotions />
            {isPickerOpen ? (
              <Picker
                data={data}
                locale="ja"
                set="iphone"
                maxFrequentRows={1}
                onEmojiSelect={(emojiData: any) => {
                  if (textArea.current) {
                    // 編集中だったら現在のカーソル位置に絵文字を挿入
                    const pos = textArea.current.selectionStart;
                    setMarkdown(markdown.slice(0, pos) + emojiData.native + markdown.slice(pos))
                  } else {
                    // 編集中以外は末尾に挿入
                    setMarkdown(markdown + emojiData.native);
                    sticker.saveMarkdown(markdown);
                  }

                  setPickerOpen(false);
                }}
              />
            ) : (
              <></>
            )}
          </button>
        </div>
        <div className="manager">
          <button
            onClick={(e) => {
              e.preventDefault();
              sticker.create();
            }}
          >
            <VscNewFile />
          </button>
          <button
            onClick={(e) => {
              e.preventDefault();
              sticker.remove();
            }}
          >
            <VscTrash />
          </button>
        </div>
      </header>
      <main>
        <Fragment>
          {editting ? (
            <textarea
              value={markdown}
              ref={textArea}
              onChange={(e) => {
                e.preventDefault();
                setMarkdown(e.target.value);
              }}
            ></textarea>
          ) : (
            <StickerMarkdown
              className="textarea"
              markdown={markdown}
              onTaskCheckChange={(task, checked) => {
                const next = markdown.replace(
                  new RegExp(`\\[[x ]\\]\\s+${task}`),
                  `[${checked ? "x" : " "}] ${task}`
                );
                setMarkdown(next);
                sticker.saveMarkdown(next);
              }}
            />
          )}
        </Fragment>
      </main>
    </div>
  );
}

export default App;
