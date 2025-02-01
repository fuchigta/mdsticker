import { invoke } from "@tauri-apps/api/core";
import "./base.css";
import "./StickerMarkdown.css";

import Markdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { solarizedDarkAtom } from "react-syntax-highlighter/dist/esm/styles/prism";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";

interface Props {
  className?: string;
  markdown: string;
  onTaskCheckChange?: (task: string, checked: boolean) => void;
}

const remarkPlugins = [remarkGfm];

function StickerMarkdown({ className, markdown, onTaskCheckChange }: Props) {
  return (
    <div
      className={className ? `StickerMarkdown ${className}` : "StickerMarkdown"}
    >
      <Markdown
        children={markdown}
        rehypePlugins={[rehypeRaw]}
        remarkPlugins={remarkPlugins}
        components={{
          code(props) {
            const { children, className, node, ref, ...rest } = props;
            const match = /language-(\w+)/.exec(className || "");
            return match ? (
              <SyntaxHighlighter
                ref={ref as React.LegacyRef<SyntaxHighlighter> | undefined}
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
          a(props) {
            const { children, href, ...rest } = props;
            return (
              <a
                {...rest}
                onClick={(e) => {
                  e.preventDefault();
                  invoke("open_url", { url: href });
                }}
              >
                {children}
              </a>
            );
          },
          input(props) {
            let { children, type, disabled, onChange, ...rest } = props;

            if (type === "checkbox" && onTaskCheckChange) {
              disabled = false;

              onChange = (e) => {
                e.preventDefault();
                const task = Array.from(e.target.parentElement!.childNodes)
                  .filter((e) => e.nodeType === Node.TEXT_NODE)
                  .map((e) => e.textContent)
                  .join("")
                  .trim();
                onTaskCheckChange(task, e.target.checked);
              };
            }

            return (
              <input
                {...rest}
                type={type}
                disabled={disabled}
                onChange={onChange}
              >
                {children}
              </input>
            );
          },
        }}
      />
    </div>
  );
}

export default StickerMarkdown;
