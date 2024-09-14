import "./base.css";
import "./StickerMarkdown.css";

import Markdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { solarizedDarkAtom } from "react-syntax-highlighter/dist/esm/styles/prism";

interface Props {
  className?: string;
  markdown: string;
}

function StickerMarkdown({ className, markdown }: Props) {
  return (
    <div className={className ? `StickerMarkdown, ${className}` : "StickerMarkdown"}>
      <Markdown
        children={markdown}
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
        }}
      />
    </div>
  );
}

export default StickerMarkdown;