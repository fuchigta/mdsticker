import { Fragment, useState } from "react";
import "./App.css";
import Markdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { solarizedDarkAtom } from "react-syntax-highlighter/dist/esm/styles/prism";
import { invoke } from "@tauri-apps/api";

const sticker = {
  async create() {
    await invoke("new_sticker");
  },
  async save(markdown: string) {
    await invoke("save_sticker", { markdown });
  },
  async remove() {
    await invoke("remove_sticker");
  },
};

function App() {
  const [editting, setEditting] = useState(false);
  const [markdown, setMarkdown] = useState("");

  return (
    <div className="container">
      <header>
        <div className="controller">
          <button
            onClick={(e) => {
              e.preventDefault();
              setEditting(!editting);
              sticker.save(markdown);
            }}
          >
            {editting ? "DONE" : "EDIT"}
          </button>
        </div>
        <div className="manager">
          <button
            onClick={(e) => {
              e.preventDefault();
              sticker.create();
            }}
          >
            NEW
          </button>
          <button
            onClick={(e) => {
              e.preventDefault();
              sticker.remove();
            }}
          >
            REMOVE
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
