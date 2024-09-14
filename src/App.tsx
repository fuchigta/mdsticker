import { Fragment, useEffect, useState } from "react";
import "./App.css";
import Markdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { solarizedDarkAtom } from "react-syntax-highlighter/dist/esm/styles/prism";
import { invoke } from "@tauri-apps/api";
import { VscEdit, VscNewFile, VscPin, VscPinned, VscSave, VscSymbolColor, VscTrash } from "react-icons/vsc";

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
            <button><VscSymbolColor /></button>
            <input
              type="color"
              onChange={(e) => {
                e.preventDefault();
                setColor(e.target.value);
                sticker.saveColor(e.target.value);
              }}
            />
          </div>
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
              onChange={(e) => {
                e.preventDefault();
                setMarkdown(e.target.value);
              }}
            ></textarea>
          ) : (
            <div className="markdown">
              <Markdown
                children={markdown}
                components={{
                  code(props) {
                    const { children, className, node, ref, ...rest } = props;
                    const match = /language-(\w+)/.exec(className || "");
                    return match ? (
                      <SyntaxHighlighter
                        ref={
                          ref as React.LegacyRef<SyntaxHighlighter> | undefined
                        }
                        {...rest}
                        PreTag="div"
                        children={String(children).replace(/\n$/, "")}
                        language={match[1]}
                        style={solarizedDarkAtom}
                      />
                    ) : (
                      <code ref={ref} {...rest} className={className}>
                        {children}
                      </code>
                    );
                  },
                }}
              />
            </div>
          )}
        </Fragment>
      </main>
    </div>
  );
}

export default App;
